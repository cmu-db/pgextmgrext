use std::path::Path;

use anyhow::Result;
use console::style;
use duct::cmd;

pub fn download_zip(zip_url: String, download_path: &Path, build_dir: &Path) -> Result<()> {
    if download_path.exists() {
        println!("{} {}", style("Skipping Download").bold().blue(), zip_url);
        return Ok(());
    }
    println!("{} {}", style("Downloading").bold().blue(), zip_url);
    let dir = tempfile::tempdir()?;
    let download_temp_path = dir.path().join("download.zip");
    cmd!("wget", zip_url, "-O", &download_temp_path).run()?;
    cmd!("mv", download_temp_path, &download_path).run()?;
    cmd!("mkdir", "-p", &build_dir).run()?;
    cmd!("unzip", &download_path, "-d", &build_dir).run()?;
    Ok(())
}

pub fn download_tar(tar_url: String, download_path: &Path, build_dir: &Path) -> Result<()> {
    if download_path.exists() {
        println!("{} {}", style("Skipping Download").bold().blue(), tar_url);
        return Ok(());
    }
    println!("{} {}", style("Downloading").bold().blue(), tar_url);
    let dir = tempfile::tempdir()?;
    let download_temp_path = dir.path().join("download.tar.gz");
    cmd!("wget", tar_url, "-O", &download_temp_path).run()?;
    cmd!("mv", download_temp_path, &download_path).run()?;
    cmd!("mkdir", "-p", &build_dir).run()?;
    cmd!("tar", "xzf", &download_path, "-C", &build_dir).run()?;
    Ok(())
}
