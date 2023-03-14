use std::path::Path;

use anyhow::{anyhow, Context, Result};
use console::style;
use duct::cmd;
use walkdir::WalkDir;

use crate::plugin::Plugin;

fn parse_pgxs_makefile(path: &Path, pg_config_key_val: &str) -> Result<String> {
    const PATCH: &str = "pgext-cli-pgxs-extension:\n\t@echo \"$(EXTENSION)\"\n";
    std::fs::write(
        path,
        patch_makefile(&std::fs::read_to_string(path)?, PATCH)?,
    )?;
    let ext_name = cmd!(
        "make",
        "pgext-cli-pgxs-extension",
        "USE_PGXS=1",
        &pg_config_key_val
    )
    .dir(path.parent().unwrap())
    .read()
    .context("failed to retrieve extension name")?;
    let ext_name = ext_name.trim().to_string();
    if ext_name.is_empty() {
        return Err(anyhow!("Failed to resolve extension name"));
    }
    Ok(ext_name)
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

pub fn resolve_build_pgxs(
    plugin: &Plugin,
    build_dir: &Path,
    pg_config: &str,
    verbose: bool,
) -> Result<()> {
    for entry in WalkDir::new(build_dir) {
        let entry = entry?;
        let path = entry.path();
        if let Some(fname) = path.file_name() {
            if fname == "Makefile" {
                println!("Found Makefile at {}", path.display());
                let parent = path.parent().unwrap();

                if parent.join("configure").exists() && !parent.join("config.status").exists() {
                    println!(
                        "{} {}",
                        style("Configure").bold().blue(),
                        plugin.resolver_args.join(" ")
                    );

                    let cmd = duct::cmd(parent.join("configure"), plugin.resolver_args.iter());
                    let cmd = if verbose {
                        cmd
                    } else {
                        cmd.stderr_null().stdout_null()
                    };
                    cmd.dir(parent).run().context("failed to configure")?;
                }

                let pg_config = format!("PG_CONFIG={}", pg_config);

                let ext_name = match parse_pgxs_makefile(path, &pg_config) {
                    Ok(ext_name) => {
                        if ext_name != plugin.name {
                            return Err(anyhow!(
                                "Extension name {} doesn't match with plugindb {}",
                                ext_name,
                                plugin.name
                            ));
                        }
                        ext_name
                    }
                    Err(err) => {
                        println!("{err}");
                        plugin.name.to_string()
                    }
                };

                println!("{} {}", style("Building").bold().blue(), ext_name);

                let num_cpus = num_cpus::get().to_string();

                if verbose {
                    cmd!("make", "USE_PGXS=1", &pg_config, "-j", num_cpus)
                } else {
                    cmd!("make", "USE_PGXS=1", &pg_config, "-j", num_cpus, "-s")
                }
                .dir(parent)
                .run()
                .context("failed to make")?;
                if verbose {
                    cmd!("make", "USE_PGXS=1", &pg_config, "install")
                } else {
                    cmd!("make", "USE_PGXS=1", &pg_config, "install", "-s")
                }
                .dir(parent)
                .run()
                .context("failed to install")?;
                return Ok(());
            }
        }
    }
    Err(anyhow!("Could not find PGXS Makefile in build dir"))
}
