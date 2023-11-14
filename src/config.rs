use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "weduck-portable-rs", about = "Command line version of weduck client implemented by Rust")]
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

    /// Run shell command by user-defined shell executor.
    /// 
    /// For example,
    ///  - for windows, by default it will act the same as -e "cmd /c".
    ///  - for Linux, the default is same as -e "bash -c"
    /// If you want to switch the executor to Powershell 7,
    ///   try: -e "pwsh -Command"
    /// 
    /// For more complicated command,
    /// it is suggested that you should wrap it as a script file to call.
    /// For example, pass no "-e" argument and set the shell remotely like
    ///   "pwsh -File C:\example.ps1"
    /// 
    /// The input text will be split by whitespace:
    ///  - the first will be the executable file name;
    ///  - the rest will be arguments following the first;
    ///  - the shell_command published by remote will be the last argument
    ///      directly passing to the first without splitting.
    #[structopt(short="e", long)]
    pub shell_executor: Option<String>,
}
