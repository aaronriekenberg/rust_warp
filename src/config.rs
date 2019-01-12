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

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn command(&self) -> &String {
        &self.command
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProxyInfo {
    api_path: String,
    html_path: String,
    description: String,
    url: String,
}

impl ProxyInfo {
    pub fn api_path(&self) -> &String {
        &self.api_path
    }

    pub fn html_path(&self) -> &String {
        &self.html_path
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn url(&self) -> &String {
        &self.url
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaticPathInfo {
    http_path: String,
    fs_path: String,
    content_type: String,
    cache_control: String,
    include_in_main_page: bool,
}

impl StaticPathInfo {
    pub fn http_path(&self) -> &String {
        &self.http_path
    }

    pub fn fs_path(&self) -> &String {
        &self.fs_path
    }

    pub fn content_type(&self) -> &String {
        &self.content_type
    }

    pub fn cache_control(&self) -> &String {
        &self.cache_control
    }

    pub fn include_in_main_page(&self) -> bool {
        self.include_in_main_page
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

impl MainPageInfo {
    pub fn title(&self) -> &String {
        &self.title
    }
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
