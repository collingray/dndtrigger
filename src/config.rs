use serde::{Deserialize, Serialize};
use plist;

static CONFIG_PATH: &str = "/Library/Preferences/sh.collin.dndtrigger.plist";

#[derive(Default)]
#[derive(Deserialize, Serialize)]
#[derive(Clone)]
pub struct DNDTriggerConfig {
    pub on_enable: Option<String>,
    pub on_disable: Option<String>,
    pub user: Option<String>,
}

// Write the configuration options to a file
pub fn write_config(on_enable: Option<&str>, on_disable: Option<&str>, user: Option<&str>) {
    let old_config: DNDTriggerConfig = plist::from_file(&CONFIG_PATH).unwrap_or_default();

    let new_config = DNDTriggerConfig {
        on_enable: on_enable.map(|s| s.to_string()).or_else(|| old_config.on_enable),
        on_disable: on_disable.map(|s| s.to_string()).or_else(|| old_config.on_disable),
        user: user.map(|s| s.to_string()).or_else(|| old_config.user).filter(|s| s != "root"),
    };

    plist::to_file_xml(&CONFIG_PATH, &new_config).expect("Failed to write config");
}

/// Load the configuration options from a file
pub fn read_config() -> DNDTriggerConfig {
    plist::from_file(&CONFIG_PATH).unwrap_or_default()
}