use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "wqy-client-rs", about = "Command line version of weduck client implemented by Rust")]
pub struct AppConfig {
    /// device id, eg. 12345678
    #[structopt(short="i", long="id")]
    pub device_id: String,

    /// device password, eg. 123456
    #[structopt(short="p", long="password")]
    pub device_password: String,

    /// Max retry times when connection lost
    #[structopt(short="r", long="max-retry", default_value="5")]
    pub max_retry_times: i32,

    /// Level in env_logger, typically could be [Error|Info|Debug].
    /// Will change your environment variable "RUST_LOG" to "log_level".
    #[structopt(short="l", long)]
    pub log_level: Option<String>,
}
