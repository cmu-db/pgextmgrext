use anyhow::{bail, Context, Result};
use console::style;
use duct::cmd;
use postgres::{Client, NoTls};

use crate::plugin::{InstallStrategy, Plugin};

/// An extension test controller trait
pub trait ExtTestControl {
  /// Connect to an postgres database for testing
  fn connect_test_db() -> Result<Client>;
  /// Handle extensions already installed in the database before testing
  fn handle_installed<F: Fn(String)>(&mut self, println: F) -> Result<()>;
  /// Display `shared_preload_libraries`
  fn show_preload_libraries<F: Fn(String)>(&mut self, println: F) -> Result<()>;
  /// Run `CREATE EXTENSION IF NOT EXISTS` for the plugin and its dependencies
  fn create_exns_for(&mut self, plugin: &Plugin) -> Result<()>;
  /// Run `DROP EXTENSION` for the plugin and its dependencies
  fn drop_exns_for<F: Fn(String)>(&mut self, plugin: &Plugin, println: F) -> Result<()>;
  /// Show all installed hooks
  fn show_hooks_all<F: Fn(String)>(&mut self, println: F) -> Result<Vec<String>>;
  /// Create an extension if not exists
  fn create_exn_if_absent(&mut self, extname: &str) -> Result<u64>;
  /// Drop an extension
  fn drop_exn(&mut self, extname: &str) -> Result<u64>;
}

impl ExtTestControl for Client {
  fn connect_test_db() -> Result<Client> {
    let whoami = cmd!("whoami").read()?;
    let client = Client::connect(
      &format!("host=localhost dbname=postgres user={} port=28815", whoami.trim()),
      NoTls,
    )?;
    Ok(client)
  }

  fn show_preload_libraries<F: Fn(String)>(&mut self, println: F) -> Result<()> {
    let result = self.query_one("SHOW shared_preload_libraries;", &[])?;
    println(format!("shared_preload_libraries: {}", result.get::<_, String>(0)));

    let result = self.query_one("SHOW session_preload_libraries;", &[])?;
    println(format!("session_preload_libraries: {}", result.get::<_, String>(0)));

    let result = self.query_one("SHOW local_preload_libraries;", &[])?;
    println(format!("local_preload_libraries: {}", result.get::<_, String>(0)));

    Ok(())
  }

  fn create_exn_if_absent(&mut self, extname: &str) -> Result<u64> {
    let n = self.execute(&format!("CREATE EXTENSION IF NOT EXISTS {};", extname), &[])?;
    Ok(n)
  }

  fn drop_exn(&mut self, extname: &str) -> Result<u64> {
    let n = self.execute(&format!("DROP EXTENSION {};", extname), &[])?;
    Ok(n)
  }

  fn create_exns_for(&mut self, plugin: &Plugin) -> Result<()> {
    for extname in plugin.dependencies.iter() {
      self.create_exn_if_absent(extname)?;
    }

    if let InstallStrategy::Install | InstallStrategy::PreloadInstall | InstallStrategy::LoadInstall =
      plugin.install_strategy
    {
      self.create_exn_if_absent(&plugin.name)?;
    }

    if let InstallStrategy::LoadInstall | InstallStrategy::Load = plugin.install_strategy {
      self
        .execute(&format!("LOAD '{}';", plugin.name), &[])
        .context("... when load extension")?;
    }
    Ok(())
  }

  fn drop_exns_for<F: Fn(String)>(&mut self, plugin: &Plugin, println: F) -> Result<()> {
    if let InstallStrategy::Install | InstallStrategy::PreloadInstall | InstallStrategy::LoadInstall =
      plugin.install_strategy
    {
      if let Err(e) = self
        .drop_exn(&plugin.name)
        .with_context(|| format!("when drop {}", plugin.name))
      {
        println(format!("{}: {}", style("Error").red().bold(), e));
      }
    }

    for extname in plugin.dependencies.iter().rev() {
      self.drop_exn(extname).context("when drop dependent extension")?;
    }
    Ok(())
  }

  fn show_hooks_all<F: Fn(String)>(&mut self, println: F) -> Result<Vec<String>> {
    let rows = self.query("SELECT * FROM show_hooks.all();", &[])?;

    let mut hooks = vec![];

    for x in rows.iter() {
      if x.get::<_, Option<String>>(1).is_some() {
        let hook_name = x.get::<_, String>(0);
        println(format!("{}: installed", hook_name));
        hooks.push(hook_name);
      }
    }
    anyhow::Ok(hooks)
  }

  fn handle_installed<F: Fn(String)>(&mut self, println: F) -> Result<()> {
    let result = self.query("SELECT extname, extversion FROM pg_extension;", &[])?;
    for x in result.iter() {
      let name = x.get::<_, String>(0);
      let ver = x.get::<_, String>(1);
      if name == "plpgsql" {
        // it's fine to keep them
        println(format!("installed pg_extension: {name}@{ver}"));
      } else if name == "pgx_show_hooks" {
        println(format!("installed pg_extension: {name}@{ver}"));
        const REQUIRED_VER: &str = "0.0.3";
        if ver != REQUIRED_VER {
          bail!("require pgx_show_hooks@{REQUIRED_VER}, but found {ver}");
        }
      } else {
        println(format!("dropping pg_extension: {name}@{ver}"));
        self.drop_exn(&name)?;
      }
    }
    Ok(())
  }
}

/// Start the pgx-managed postgres 15 instance
pub fn pgx_start_pg15() -> Result<()> {
  let output = cmd!("cargo", "pgrx", "start", "pg15")
    .dir("pgx_show_hooks")
    .stderr_to_stdout()
    .stdout_capture()
    .unchecked()
    .run()?;

  if !output.status.success() {
    println!("{}", std::str::from_utf8(&output.stdout)?);
    let log = home::home_dir().unwrap().join(".pgrx").join("15.log"); // TODO: pgx should support this
    cmd!("tail", "-n", "50", log).run()?;
    return Err(anyhow::anyhow!("Failed to start pg15"));
  }
  Ok(())
}

/// Stop the pgx-managed postgres 15 instance
pub fn pgx_stop_pg15() -> Result<()> {
  cmd!("cargo", "pgrx", "stop", "pg15")
    .dir("pgx_show_hooks")
    .stderr_null()
    .stdout_null()
    .run()?;
  Ok(())
}
