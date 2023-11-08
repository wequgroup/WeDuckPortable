use serde::{Serialize, Deserialize};
use std::fs;
use toml;

pub const CONFIG_PATH: &str = "./config.toml";
type AnyError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_id: String,
    pub device_password: String,
    pub retry_times: i32,
}

pub fn from_file(path: &str) -> Result<AppConfig, AnyError> {
    Ok(toml::from_str(
        fs::read_to_string(path)?.as_str()
    )?)
    
}

/// 保存当前实例为配置文件，会直接覆盖。
pub fn save_demo() -> Result<(), AnyError> {
    let demo = AppConfig {
        device_id: "12345678".to_string(),
        device_password: "123456".to_string(),
        retry_times: -1
    };
    let toml_str = toml::to_string(&demo)?;
    fs::write(CONFIG_PATH, &toml_str)?;
    Ok(())
}