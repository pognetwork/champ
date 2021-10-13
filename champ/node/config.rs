use anyhow::Result;
use clap::ArgMatches;
use std::fs::OpenOptions;
use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {}

pub fn new(cli_args: ArgMatches) -> Result<Config> {
    let config_file = if let Some(c) = cli_args.value_of("config") {
        std::fs::canonicalize(c)?
    } else {
        let project_dir =
            directories::ProjectDirs::from("network", "pog", "champ").expect("failed to create project dir");
        let config_dir = project_dir.config_dir();
        std::fs::create_dir_all(config_dir)?;
        config_dir.join("champ.toml")
    };

    let mut file = String::new();
    OpenOptions::new().write(true).create(true).open(config_file)?.read_to_string(&mut file)?;
    let config = toml::from_str::<Config>(&file)?;

    Ok(config)
}
