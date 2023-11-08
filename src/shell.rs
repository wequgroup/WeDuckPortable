use std::{process::{Command, Output}, io::{Error, ErrorKind}};
use log::{debug, info, error};

fn display_output(o: &Output) -> String {
    format!(
        "Output(ExitStatus = {}, stdout = \"{}\", stderr = \"{}\")",
        o.status,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// 尝试从Publish包中解析出shell command，同时检查shell type是否支持，失败返回空。
pub fn extract_command<'a>(json: &'a serde_json::Value) -> Option<&'a str> {
    // 检测数据是否支持
    let shell_type = json.get("shellType")?.as_i64()?;
    if shell_type != 0 {
        error!("Not supported shell type: {}", shell_type);
        return None;
    };
    
    json.get("shellContent")?.as_str()
}

/// 根据不同平台进行不同的shell调用。
pub fn shell_runner(command: &str) {
    debug!("Shell run: {}", command);

    let out = if cfg!(windows) {
        Command::new("cmd")
            .arg("/c")
            .arg(command)
            .output()
    } else if cfg!(unix) {
        Err(Error::new(ErrorKind::Other, "Not Supported System"))
    } else {
        Err(Error::new(ErrorKind::Other, "Not Supported System"))
    };
    
    match out {
        Ok(o) => info!("Shell output: {}", display_output(&o)),
        Err(e) => error!("Shell error: {}", e)
    }
}