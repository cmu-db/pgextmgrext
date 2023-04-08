use std::path::Path;

use anyhow::Result;
use console::style;
use duct::cmd;

use crate::plugin::GitDownload;

/// Download and uncompress a zip file
pub fn download_zip(zip_url: String, download_path: &Path, build_dir: &Path, verbose: bool) -> Result<()> {
  if download_path.exists() {
    println!("{} {}", style("Skipping Download").bold().blue(), zip_url);
  } else {
    println!("{} {}", style("Downloading").bold().blue(), zip_url);
    let dir = tempfile::tempdir()?;
    let download_temp_path = dir.path().join("download.zip");

    if verbose {
      cmd!("wget", zip_url, "-O", &download_temp_path)
    } else {
      cmd!("wget", zip_url, "-O", &download_temp_path, "-q")
    }
    .run()?;

    cmd!("mv", download_temp_path, &download_path).run()?;
  }

  cmd!("mkdir", "-p", &build_dir).run()?;

  let cmd = cmd!("unzip", "-o", &download_path, "-d", &build_dir);
  let cmd = if verbose { cmd } else { cmd.stderr_null().stdout_null() };
  cmd.run()?;

  Ok(())
}

/// Download an uncompress a tar file
pub fn download_tar(tar_url: String, download_path: &Path, build_dir: &Path, verbose: bool) -> Result<()> {
  if download_path.exists() {
    println!("{} {}", style("Skipping Download").bold().blue(), tar_url);
  } else {
    println!("{} {}", style("Downloading").bold().blue(), tar_url);
    let dir = tempfile::tempdir()?;
    let download_temp_path = dir.path().join("download.tar.gz");

    if verbose {
      cmd!("wget", tar_url, "-O", &download_temp_path)
    } else {
      cmd!("wget", tar_url, "-O", &download_temp_path, "-q")
    }
    .run()?;

    cmd!("mv", download_temp_path, &download_path).run()?;
  }

  cmd!("mkdir", "-p", &build_dir).run()?;

  let cmd = cmd!("tar", "xzf", &download_path, "-C", &build_dir);
  let cmd = if verbose { cmd } else { cmd.stderr_null().stdout_null() };
  cmd.run()?;

  Ok(())
}

pub fn download_git(src: &GitDownload, build_dir: &Path, verbose: bool) -> Result<()> {
  if build_dir.exists() {
    println!("{} {}", style("Skipping Download").bold().blue(), src.url);
  } else {
    println!("{} {}", style("Clone").bold().blue(), src.url);
    if verbose {
      cmd!("git", "clone", &src.url, &build_dir)
    } else {
      cmd!("git", "clone", &src.url, &build_dir, "--quiet")
    }
    .run()?;
  }

  cmd!("git", "reset", "--hard").dir(build_dir).run()?;
  if let Some(rev) = &src.rev {
    cmd!("git", "checkout", rev).dir(build_dir).run()?;
  }

  Ok(())
}
