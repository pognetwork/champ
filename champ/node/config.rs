use crate::storage::{DatabaseConfig, Databases};
use anyhow::Result;
use anyhow::{anyhow, Context};
use clap::ArgMatches;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

fn default_accounts() -> BTreeMap<String, UserAccount> {
    BTreeMap::new()
}

fn default_admin() -> Admin {
    Admin {
        enabled: false,
        ..Default::default()
    }
}

fn default_database() -> DatabaseConfig {
    DatabaseConfig {
        kind: Databases::Sled,
        ..Default::default()
    }
}

fn default_node_name() -> String {
    "PogNetwork Node".to_string()
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default = "default_admin")]
    pub admin: Admin,

    #[serde(default = "default_accounts", serialize_with = "toml::ser::tables_last")]
    pub admin_accounts: BTreeMap<String, UserAccount>,

    #[serde(default = "default_database")]
    pub database: DatabaseConfig,

    #[serde(skip_serializing)]
    config_path_override: Option<String>,

    #[serde(skip_serializing)]
    pub data_path: Option<String>,

    #[serde(skip_serializing)]
    pub config_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAccount {
    pub user_id: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Admin {
    pub enabled: bool,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    #[serde(default = "default_node_name")]
    pub node_name: String,
}

impl Config {
    fn get_default_data_path() -> Result<PathBuf> {
        let project_dir = directories::ProjectDirs::from("network", "pog", "champ").expect("failed to create data dir");
        let data_dir = project_dir.data_dir();
        std::fs::create_dir_all(data_dir)?;
        Ok(data_dir.to_path_buf())
    }

    pub fn get_path(&self) -> Result<PathBuf> {
        let path = if let Some(config_path) = &self.config_path_override {
            let path: PathBuf = config_path.parse()?;
            path.absolutize()?.into()
        } else {
            let project_dir =
                directories::ProjectDirs::from("network", "pog", "champ").expect("failed to create project dir");
            let config_dir = project_dir.config_dir();
            std::fs::create_dir_all(config_dir)?;
            config_dir.join("champ.toml")
        };

        Ok(path)
    }

    pub fn new(cli_args: Option<ArgMatches>) -> Result<Self> {
        let mut config = Config {
            config_path_override: cli_args.map_or_else(|| None, |a| a.value_of("config").map(|x| x.to_string())),
            ..Default::default()
        };

        config.read()?;
        config.write()?;
        Ok(config)
    }

    pub fn read(&mut self) -> Result<()> {
        let config_path = self.get_path()?;
        let config_file = read_or_create_file(config_path.clone()).with_context(|| "failed to read file")?;
        let config = toml::from_str::<Config>(&config_file).with_context(|| "failed to parse file")?;
        let config_path = config_path.as_path();

        // Update config
        self.config_path = config_path.to_str().map(|p| p.to_string());
        self.database = config.database.clone();
        self.admin = config.admin;
        self.admin_accounts = config.admin_accounts;

        self.data_path = if let Some(path) = config.database.path {
            let path = path.parse::<PathBuf>()?;
            Some(
                path.absolutize_from(config_path)?
                    .to_str()
                    .ok_or_else(|| anyhow!("unknown database path"))?
                    .to_string(),
            )
        } else {
            Config::get_default_data_path()?.to_str().map(|p| p.to_string())
        };
        self.database.data_path = self.data_path.clone();
        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        let config_path = self.get_path()?;
        let config = toml::to_string_pretty::<Config>(self)?;
        write_file(config_path, &config)?;
        Ok(())
    }
}

pub fn read_or_create_file(path: PathBuf) -> Result<String> {
    let mut file = String::new();
    let mut f = OpenOptions::new().read(true).write(true).create(true).open(path)?;
    f.read_to_string(&mut file).expect("should read file to string");
    drop(f);
    Ok(file)
}

pub fn write_file(path: PathBuf, data: &str) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(path)?;
    file.write_all(data.as_bytes())?;
    drop(file);
    Ok(())
}
