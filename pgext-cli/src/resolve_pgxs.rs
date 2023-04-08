//! Helper functions for working with PGXS

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use console::style;
use duct::cmd;
use walkdir::WalkDir;

use crate::plugin::Plugin;

/// Parse PGXS Makefile to get extension name
fn parse_pgxs_makefile(path: &Path, pg_config_key_val: &str) -> Result<String> {
  const PATCH: &str = "pgext-cli-pgxs-extension:\n\t@echo \"$(EXTENSION)\"\n";
  std::fs::write(path, patch_makefile(&std::fs::read_to_string(path)?, PATCH)?)?;
  let ext_name = cmd!("make", "pgext-cli-pgxs-extension", "USE_PGXS=1", &pg_config_key_val)
    .dir(path.parent().unwrap())
    .read()
    .context("failed to retrieve extension name")?;
  let ext_name = ext_name.trim().to_string();
  if ext_name.is_empty() {
    return Err(anyhow!("Failed to resolve extension name"));
  }
  Ok(ext_name)
}

/// Make patches to the PGXS Makefile
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
    return Ok(format!("{}{}{}{}", content, PGEXT_CLI_BEGIN, patch, PGEXT_CLI_END));
  }
}

/// Find PGXS Makefile Path
pub fn find_pgxs_path(build_dir: &Path) -> Result<Option<PathBuf>> {
  let mut final_path: Option<PathBuf> = None;
  for entry in WalkDir::new(build_dir) {
    let entry = entry?;
    let path = entry.path();
    if let Some(fname) = path.file_name() {
      if fname == "Makefile" {
        println!("Found Makefile at {}", path.display());
        if let Some(final_path) = final_path.as_mut() {
          let new_path = path.parent().unwrap();
          if final_path.starts_with(new_path) {
            *final_path = new_path.to_path_buf();
          }
        } else {
          final_path = Some(path.parent().unwrap().to_path_buf());
        }
      }
    }
  }
  Ok(final_path)
}

/// Patches Makefile to load two extensions for regression tests
fn pgxs_regress_load_extensions(path: &Path, extnames: Option<&Vec<String>>) -> Result<()> {
  const REGRESS_OPTS: &str = "REGRESS_OPTS += --load-extension";
  if let Some(names) = extnames {
    let patch = names
      .iter()
      .map(|name| format!("{} {:#?}\n", REGRESS_OPTS, name))
      .collect::<Vec<String>>()
      .join("\n");
    std::fs::write(path, patch_makefile(&std::fs::read_to_string(path)?, &patch)?)?;
  } else {
    std::fs::write(path, patch_makefile(&std::fs::read_to_string(path)?, "")?)?;
  }
  Ok(())
}

/// Force use GNU sed on macOS
fn add_gnu_sed(mut cmd: duct::Expression) -> Result<duct::Expression> {
  if cfg!(target_os = "macos") {
    let homebrew_prefix = std::env::var("HOMEBREW_PREFIX").context("homebrew prefix not found in env var")?;
    let path = std::env::var("PATH")?;
    let gnu_path = format!("{}/opt/gnu-sed/libexec/gnubin", homebrew_prefix);
    if !Path::new(&gnu_path).join("sed").try_exists()? {
      bail!("Please install gnu-sed by: brew install gnu-sed");
    }
    let path = format!("{}:{}", gnu_path, path);
    cmd = cmd.env("PATH", path);
  }
  Ok(cmd)
}

/// Run PGXS `make installcheck` command
pub fn pgxs_installcheck(
  plugin: &Plugin,
  other: Option<(&Vec<String>, &Vec<String>)>,
  build_dir: &Path,
  pg_config: &str,
) -> Result<()> {
  let pg_host = home::home_dir().unwrap().join(".pgx");
  let final_path = if plugin.resolver.as_str() != "pgsrctree" {
    find_pgxs_path(build_dir)?
  } else {
    Some(pg_host.join("15.2/contrib").join(plugin.name.clone()))
  };

  if let Some(parent) = final_path {
    let path = parent.join("Makefile");
    pgxs_regress_load_extensions(&path, other.map(|x| x.0))?;

    let pg_config = format!("PG_CONFIG={}", pg_config);
    let whoami = cmd!("whoami").read()?;
    let pg_user = format!("PG_USER={}", whoami.trim());
    let pg_host = format!("PGHOST={}", pg_host.to_str().unwrap());
    add_gnu_sed(cmd!("make", "-B", "USE_PGXS=1", pg_user, pg_config, pg_host, "installcheck").dir(parent))?
      .run()
      .context(format!(
        "error when running `make installcheck` on {}",
        style(&plugin.name).bold()
      ))?;
    Ok(())
  } else {
    Err(anyhow!("Could not find PGXS Makefile in build dir"))
  }
}

/// Resolve and build extensions using PGXS Makefile
pub fn resolve_build_pgxs(plugin: &Plugin, build_dir: &Path, pg_config: &str, verbose: bool) -> Result<()> {
  let final_path: Option<PathBuf> = find_pgxs_path(build_dir)?;

  if let Some(parent) = final_path {
    if parent.join("configure").exists() && !parent.join("config.status").exists() {
      println!(
        "{} {}",
        style("Configure").bold().blue(),
        plugin.resolver_args.join(" ")
      );

      let cmd = duct::cmd(parent.join("configure"), plugin.resolver_args.iter());
      let cmd = if verbose { cmd } else { cmd.stderr_null().stdout_null() };
      cmd
        .dir(&parent)
        .env("PG_CONFIG", pg_config)
        .run()
        .context("failed to configure")?;
    }

    let pg_config = format!("PG_CONFIG={}", pg_config);

    let path = parent.join("Makefile");
    let ext_name = match parse_pgxs_makefile(&path, &pg_config) {
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
    .dir(&parent)
    .run()
    .context("failed to make")?;
    if verbose {
      cmd!("make", "USE_PGXS=1", &pg_config, "install")
    } else {
      cmd!("make", "USE_PGXS=1", &pg_config, "install", "-s")
    }
    .dir(&parent)
    .run()
    .context("failed to install")?;
    Ok(())
  } else {
    Err(anyhow!("Could not find PGXS Makefile in build dir"))
  }
}
