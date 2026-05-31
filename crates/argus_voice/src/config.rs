use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct ArgusConfig {
    pub bookmarks: HashMap<String, String>,
}

pub fn load_config() -> ArgusConfig {
    // Look for the config file in the root directory
    let config_str = fs::read_to_string("argus.toml")
        .expect("CRITICAL ERROR: Could not find argus.toml in the root directory.");
    
    // Parse the TOML text into our Rust struct
    toml::from_str(&config_str)
        .expect("CRITICAL ERROR: Your argus.toml file has a syntax error.")
}