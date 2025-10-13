use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::fs;
use std::process::exit;

const DEFAULT_PORT: i32 = 8796;
const DEFAULT_STORAGE: &str = ".xchg/storage";
const DEFAULT_ADDRESS: &str = "/ip4/0.0.0.0/tcp/0";

const DEFAULT_LEVEL: &str = "info";
const DEFAULT_LOGS_PATH: &str = ".xchg/logs";

#[derive(Clone, Serialize, Deserialize)]
pub struct Logs {
    level: Option<String>,
    dir: Option<String>,
}

impl Default for Logs {
    fn default() -> Self {
        let path = get_path(DEFAULT_LOGS_PATH);
        Self {
            level: Some(String::from(DEFAULT_LEVEL)),
            dir: Some(path),
        }
    }
}

impl Logs {
    pub fn level(&self) -> String {
        self.level.clone().unwrap()
    }

    pub fn dir(&self) -> String {
        self.dir.clone().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    http_port: Option<i32>,
    address: Option<String>,
    logs: Option<Logs>,
    secret: String,
    storage_path: Option<String>,
    nodes: Option<Vec<String>>,
}

impl Config {
    pub fn read(path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str::<Config>(&json) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("{:?}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
                exit(1);
            }
        }
    }

    pub fn http_port(&self) -> i32 {
        self.http_port.unwrap_or(DEFAULT_PORT)
    }

    pub fn address(&self) -> String {
        if let Some(address) = &self.address {
            address.clone()
        } else {
            DEFAULT_ADDRESS.to_string()
        }
    }

    pub fn nodes(&self) -> Vec<String> {
        if let Some(nodes) = &self.nodes {
            nodes.clone()
        } else {
            Vec::new()
        }
    }

    pub fn secret(&self) -> String {
        self.secret.clone()
    }

    pub fn storage_path(&self) -> String {
        if let Some(storage_path) = &self.storage_path {
            storage_path.clone()
        } else {
            get_path(DEFAULT_STORAGE)
        }
    }

    pub fn logs(&self) -> Logs {
        if let Some(logs) = self.logs.as_ref() {
            logs.clone()
        } else {
            Logs::default()
        }
    }
}

fn get_path(path: &str) -> String {
    let path = home_dir().unwrap().join(path);
    path.to_str().unwrap().to_string()
}
