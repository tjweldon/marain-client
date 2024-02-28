use homedir::get_my_home;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use tokio::fs::create_dir_all;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    username: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self { username: None }
    }
}

const CLIENT_CONF_PATH_ENV_VAR: &str = "MARAIN_CONFIG_PATH";

pub fn config_path() -> PathBuf {
    match std::env::var(CLIENT_CONF_PATH_ENV_VAR) {
        Ok(p) => p.into(),
        _ => match get_my_home() {
            Ok(Some(p)) => p
                .join(PathBuf::from(".config"))
                .join(PathBuf::from("marain_config.json")),
            _ => current_dir().expect("Falling back to current directory for user config failed"),
        },
    }
}

pub async fn load_config() -> UserConfig {
    let conf_path = config_path();
    return if conf_path.exists() {
        read_config(&conf_path)
    } else {
        write_default_config(&conf_path).await;
        UserConfig::default()
    };
}

fn read_config(conf_path: &PathBuf) -> UserConfig {
    let contents = read_to_string(conf_path).expect(&format!(
        "Failed to read config at path: {}",
        conf_path.display()
    ));
    serde_json::from_str(&contents).expect(&format!(
        "Config file at {} schema was not valid",
        conf_path.display()
    ))
}

async fn write_default_config(conf_path: &PathBuf) {
    create_dir_all(
        &conf_path
            .parent()
            .expect(&format!("Invalid conf path: {}", conf_path.display())),
    )
    .await
    .expect(&format!(
        "Could not create config directory {}",
        conf_path.display()
    ));

    let mut file = File::create(conf_path).expect(&format!(
        "Couldn't create config file at path {}",
        conf_path.display()
    ));
    file.write_all(
        serde_json::to_string_pretty(&UserConfig::default())
            .expect("Could not serialize default config json")
            .as_bytes(),
    )
    .expect(&format!(
        "Could not write default config to path: {}",
        conf_path.display()
    ));
}
