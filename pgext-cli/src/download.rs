use std::path::Path;

use anyhow::Result;
use console::style;
use duct::cmd;

pub fn download_zip(
    zip_url: String,
    download_path: &Path,
    build_dir: &Path,
    verbose: bool,
) -> Result<()> {
    if download_path.exists() {
        println!("{} {}", style("Skipping Download").bold().blue(), zip_url);
        return Ok(());
    }
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
    cmd!("mkdir", "-p", &build_dir).run()?;

    let cmd = cmd!("unzip", &download_path, "-d", &build_dir);
    let cmd = if verbose {
        cmd
    } else {
        cmd.stderr_null().stdout_null()
    };
    cmd.run()?;

    Ok(())
}

pub fn download_tar(
    tar_url: String,
    download_path: &Path,
    build_dir: &Path,
    verbose: bool,
) -> Result<()> {
    if download_path.exists() {
        println!("{} {}", style("Skipping Download").bold().blue(), tar_url);
        return Ok(());
    }
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
    cmd!("mkdir", "-p", &build_dir).run()?;

    let cmd = cmd!("tar", "xzf", &download_path, "-C", &build_dir);
    let cmd = if verbose {
        cmd
    } else {
        cmd.stderr_null().stdout_null()
    };
    cmd.run()?;

    Ok(())
}
