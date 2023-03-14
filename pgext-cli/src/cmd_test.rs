use std::path::PathBuf;
use std::{fmt::Write, iter};

use anyhow::Result;
use console::style;
use duct::cmd;
use postgres::{Client, NoTls};

use crate::{config::load_workspace_config, plugin::load_plugin_db, CmdTest, CmdTestAll};

pub fn cmd_test_all(_: CmdTestAll) -> Result<()> {
    let db = load_plugin_db()?;
    for plugin in db.plugins {
        if let Err(e) = cmd_test(CmdTest { name: plugin.name }) {
            println!("{}: {}", style("Error").red().bold(), e);
        }
    }
    Ok(())
}

pub fn cmd_test(cmd: CmdTest) -> Result<()> {
    let db = load_plugin_db()?;
    let config = load_workspace_config()?;

    let plugin = if let Some(plugin) = db.plugins.iter().find(|x| x.name == cmd.name) {
        plugin.clone()
    } else {
        anyhow::bail!("Plugin not found");
    };

    println!(
        "{} {}",
        style("Testing").blue().bold(),
        style(&plugin.name).bold()
    );

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

    client.execute("CREATE EXTENSION IF NOT EXISTS pgx_show_hooks;", &[])?;

    for extname in plugin.dependencies.iter().chain(iter::once(&plugin.name)) {
        client.execute(&format!("CREATE EXTENSION IF NOT EXISTS {};", extname), &[])?;
    }

    let rows = client.query("SELECT * FROM show_hooks.all();", &[])?;
    rows.iter().for_each(|x| {
        if x.get::<_, Option<String>>(1).is_some() {
            println!("{}: installed", x.get::<_, String>(0),);
        }
    });

    for extname in plugin
        .dependencies
        .iter()
        .chain(iter::once(&plugin.name))
        .rev()
    {
        client.execute(&format!("DROP EXTENSION {};", extname), &[])?;
    }

    cmd!("cargo", "pgx", "stop", "pg15")
        .dir("pgx_show_hooks")
        .run()?;
    Ok(())
}
