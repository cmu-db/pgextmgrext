use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use console::style;
use duct::cmd;
use regex::Regex;
use walkdir::WalkDir;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install(CmdInstall),
}

/// Install extension
#[derive(Parser, Debug)]
pub struct CmdInstall {
    /// The URL to install from
    url: String,
    /// The path to `pg_config` binary
    pg_config: String,
}

fn parse_pgxs_makefile(f: String) -> Result<String> {
    let regex = Regex::new("EXTENSION\\s*=\\s*(.*)\\s*")?;
    for line in f.lines() {
        if let Some(cap) = regex.captures(line) {
            return Ok(cap[1].to_string());
        }
    }
    Err(anyhow!("Could not find EXTENSION in Makefile"))
}

fn do_install(cmd: CmdInstall) -> Result<()> {
    println!("{} {}", style("Downloading").bold().blue(), cmd.url);
    let dir = tempfile::tempdir()?;
    let download_path = dir.path().join("ext.tar.gz");
    let extract_path = dir.path().join("ext");
    cmd!("wget", cmd.url, "-O", &download_path).run()?;
    cmd!("mkdir", "-p", &extract_path).run()?;
    cmd!("tar", "xzf", &download_path, "-C", &extract_path).run()?;
    for entry in WalkDir::new(&extract_path) {
        let entry = entry?;
        let path = entry.path();
        if let Some(fname) = path.file_name() {
            if fname == "Makefile" {
                let ext_name = parse_pgxs_makefile(std::fs::read_to_string(path)?)?;
                println!("{} {}", style("Installing").bold().blue(), ext_name);
                let parent = path.parent().unwrap();
                let num_cpus = num_cpus::get().to_string();
                let pg_config = format!("PG_CONFIG={}", cmd.pg_config);
                cmd!("make", "-j", num_cpus, "-C", parent, "install", pg_config).run()?;
                println!("{}", style("Succeed").bold().green());
                return Ok(());
            }
        }
    }
    Err(anyhow!("no extension found in the package"))
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Install(cmd) => {
            do_install(cmd)?;
        }
    }
    Ok(())
}
