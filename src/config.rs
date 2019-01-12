use log::info;

use serde_derive::{Deserialize, Serialize};

use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    id: String,
    description: String,
    command: String,
    args: Vec<String>,
}

impl CommandInfo {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn command(&self) -> &String {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    id: String,
    description: String,
    url: String,
}

impl ProxyInfo {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn url(&self) -> &String {
        &self.url
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    listen_address: String,
}

impl ServerInfo {
    pub fn listen_address(&self) -> &String {
        &self.listen_address
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainPageInfo {
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    server_info: ServerInfo,
    main_page_info: MainPageInfo,
    commands: Vec<CommandInfo>,
    proxies: Vec<ProxyInfo>,
}

impl Configuration {
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    pub fn main_page_info(&self) -> &MainPageInfo {
        &self.main_page_info
    }

    pub fn commands(&self) -> &Vec<CommandInfo> {
        &self.commands
    }

    pub fn proxies(&self) -> &Vec<ProxyInfo> {
        &self.proxies
    }
}

pub fn read_config(config_file: String) -> Result<Configuration, Box<::std::error::Error>> {
    info!("reading {}", config_file);

    let mut file = ::std::fs::File::open(config_file)?;

    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents)?;

    let configuration: Configuration = ::serde_json::from_str(&file_contents)?;

    Ok(configuration)
}
