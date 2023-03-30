use anyhow::{bail, Context, Result};
use console::style;
use duct::cmd;
use postgres::{Client, NoTls};

use crate::plugin::{InstallStrategy, Plugin};

pub trait ExtTestControl {
  fn connect_test_db() -> Result<Client>;
  fn handle_installed<F: Fn(String)>(&mut self, println: F) -> Result<()>;
  fn show_preload_libraries<F: Fn(String)>(&mut self, println: F) -> Result<()>;
  fn create_exns_for(&mut self, plugin: &Plugin) -> Result<()>;
  fn drop_exns_for<F: Fn(String)>(&mut self, plugin: &Plugin, println: F) -> Result<()>;
  fn show_hooks_all<F: Fn(String)>(&mut self, println: F) -> Result<Vec<String>>;
  fn create_exn_if_absent(&mut self, extname: &str) -> Result<u64>;
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

pub fn pgx_start_pg15() -> Result<()> {
  let output = cmd!("cargo", "pgx", "start", "pg15")
    .dir("pgx_show_hooks")
    .stderr_to_stdout()
    .stdout_capture()
    .unchecked()
    .run()?;

  if !output.status.success() {
    println!("{}", std::str::from_utf8(&output.stdout)?);
    let log = home::home_dir().unwrap().join(".pgx").join("15.log"); // TODO: pgx should support this
    cmd!("tail", "-n", "50", log).run()?;
    return Err(anyhow::anyhow!("Failed to start pg15"));
  }
  Ok(())
}

pub fn pgx_stop_pg15() -> Result<()> {
  cmd!("cargo", "pgx", "stop", "pg15")
    .dir("pgx_show_hooks")
    .stderr_null()
    .stdout_null()
    .run()?;
  Ok(())
}
