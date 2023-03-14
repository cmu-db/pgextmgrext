use std::path::Path;

use anyhow::{anyhow, Context, Result};
use console::style;
use duct::cmd;
use walkdir::WalkDir;

use crate::plugin::Plugin;

fn parse_pgxs_makefile(path: &Path) -> Result<String> {
    const PATCH: &str = "pgext-cli-pgxs-extension:\n\t@echo \"$(EXTENSION)\"\n";
    std::fs::write(
        path,
        patch_makefile(&std::fs::read_to_string(path)?, PATCH)?,
    )?;
    let ext_name = cmd!("make", "pgext-cli-pgxs-extension")
        .dir(path.parent().unwrap())
        .read()
        .context("failed to retrieve extension name")?;
    Ok(ext_name.trim().to_string())
}

fn patch_makefile(content: &str, patch: &str) -> Result<String> {
    const PGEXT_CLI_BEGIN: &str = "\n# *****BEGIN PGEXT-CLI***** #\n";
    const PGEXT_CLI_END: &str = "\n# *****END PGEXT-CLI***** #\n";
    if content.contains(PGEXT_CLI_BEGIN) {
        let (before, rest) = content.split_once(PGEXT_CLI_BEGIN).unwrap();
        let (_, after) = rest.split_once(PGEXT_CLI_END).unwrap();
        return Ok(format!(
            "{}{}{}{}{}",
            before, PGEXT_CLI_BEGIN, patch, PGEXT_CLI_END, after
        ));
    } else {
        return Ok(format!(
            "{}{}{}{}",
            content, PGEXT_CLI_BEGIN, patch, PGEXT_CLI_END
        ));
    }
}

pub fn resolve_build_pgxs(plugin: &Plugin, build_dir: &Path, pg_config: &str) -> Result<()> {
    for entry in WalkDir::new(build_dir) {
        let entry = entry?;
        let path = entry.path();
        if let Some(fname) = path.file_name() {
            if fname == "Makefile" {
                println!("Found Makefile at {}", path.display());
                let parent = path.parent().unwrap();

                if parent.join("configure").exists() {
                    println!(
                        "{} {}",
                        style("Configure").bold().blue(),
                        plugin.resolver_args.join(" ")
                    );
                    duct::cmd(parent.join("configure"), plugin.resolver_args.iter())
                        .dir(parent)
                        .run()
                        .context("failed to configure")?;
                }

                let ext_name = parse_pgxs_makefile(path)?;
                println!("{} {}", style("Building").bold().blue(), ext_name);
                if ext_name != plugin.name {
                    return Err(anyhow!(
                        "Extension name {} doesn't match with plugindb {}",
                        ext_name,
                        plugin.name
                    ));
                }
                let num_cpus = num_cpus::get().to_string();
                let pg_config = format!("PG_CONFIG={}", pg_config);

                cmd!("make", "-j", num_cpus, "install", pg_config, "USE_PGXS=1")
                    .dir(parent)
                    .run()
                    .context("failed to make")?;
                return Ok(());
            }
        }
    }
    Err(anyhow!("Could not find PGXS Makefile in build dir"))
}
