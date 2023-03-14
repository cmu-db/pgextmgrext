use std::fmt::Write;
use std::path::PathBuf;

use anyhow::Result;
use duct::cmd;
use postgres::{Client, NoTls};

use crate::{config::load_workspace_config, plugin::load_plugin_db, CmdTest};

pub fn cmd_test(cmd: CmdTest) -> Result<()> {
    let db = load_plugin_db()?;
    let config = load_workspace_config()?;
    let plugin = if let Some(plugin) = db.plugins.iter().find(|x| x.name == cmd.name) {
        plugin.clone()
    } else {
        anyhow::bail!("Plugin not found");
    };
    cmd!("cargo", "pgx", "stop", "pg15")
        .dir("pgx_show_hooks")
        .run()?;
    {
        let conf = PathBuf::from(&config.pg_data).join("postgresql.conf");
        let pgconf = std::fs::read_to_string(&conf)?;
        let mut new_pgconf = String::new();
        for line in pgconf.lines() {
            if line.starts_with("shared_preload_libraries = ") {
                if plugin.require_shared_preload_library {
                    writeln!(
                        new_pgconf,
                        "shared_preload_libraries = '{}' # modified by pgext",
                        plugin.name
                    )?;
                } else {
                    writeln!(
                        new_pgconf,
                        "shared_preload_libraries = ''  # modified by pgext"
                    )?;
                }
            } else {
                writeln!(new_pgconf, "{}", line)?;
            }
        }
        std::fs::write(conf, new_pgconf)?;
    }
    cmd!("cargo", "pgx", "start", "pg15")
        .dir("pgx_show_hooks")
        .run()?;
    let whoami = cmd!("whoami").read()?;
    let mut client = Client::connect(
        &format!(
            "host=localhost dbname=postgres user={} port=28815",
            whoami.trim()
        ),
        NoTls,
    )?;
    let result = client.query_one("SHOW shared_preload_libraries;", &[])?;
    println!("shared_preload_libraries: {}", result.get::<_, String>(0));

    let result = client.query("SELECT extname, extversion FROM pg_extension;", &[])?;
    result.iter().for_each(|x| {
        println!(
            "installed_pg_extension: {}@{}",
            x.get::<_, String>(0),
            x.get::<_, String>(1)
        );
    });

    client.execute(
        "CREATE EXTENSION IF NOT EXISTS pgx_show_hooks;",
        &[],
    )?;

    client.execute(
        &format!("CREATE EXTENSION IF NOT EXISTS {};", plugin.name),
        &[],
    )?;

    let rows = client.query("SELECT * FROM show_hooks.all();", &[])?;
    rows.iter().for_each(|x| {
        println!(
            "{}: {}",
            x.get::<_, String>(0),
            x.get::<_, Option<String>>(1).is_some()
        );
    });

    client.execute(&format!("DROP EXTENSION {};", plugin.name), &[])?;

    cmd!("cargo", "pgx", "stop", "pg15")
        .dir("pgx_show_hooks")
        .run()?;
    Ok(())
}
