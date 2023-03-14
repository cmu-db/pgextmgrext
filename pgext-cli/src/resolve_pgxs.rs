use std::path::Path;

use anyhow::{anyhow, Result};
use console::style;
use duct::cmd;
use regex::Regex;
use walkdir::WalkDir;

fn parse_pgxs_makefile(f: String) -> Result<String> {
    let regex = Regex::new("EXTENSION\\s*=\\s*(.*)\\s*")?;
    for line in f.lines() {
        if let Some(cap) = regex.captures(line) {
            return Ok(cap[1].to_string());
        }
    }
    Err(anyhow!("Could not find EXTENSION in Makefile"))
}

pub fn resolve_build_pgxs(build_dir: &Path, pg_config: &str) -> Result<()> {
    for entry in WalkDir::new(build_dir) {
        let entry = entry?;
        let path = entry.path();
        if let Some(fname) = path.file_name() {
            if fname == "Makefile" {
                let ext_name = parse_pgxs_makefile(std::fs::read_to_string(path)?)?;
                println!("{} {}", style("Building").bold().blue(), ext_name);
                let parent = path.parent().unwrap();
                let num_cpus = num_cpus::get().to_string();
                let pg_config = format!("PG_CONFIG={}", pg_config);
                cmd!("make", "-j", num_cpus, "install", pg_config)
                    .dir(parent)
                    .run()?;
                return Ok(());
            }
        }
    }
    Err(anyhow!("Could not find PGXS Makefile in build dir"))
}
