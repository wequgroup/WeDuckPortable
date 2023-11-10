use std::process::{Command, Child};
use log::{debug, info, error};

use crate::CONFIG;

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
/// 使用全局变量：CONFIG.shell_executor
pub fn shell_runner(command: &str) -> Option<tokio::task::JoinHandle<()>> {

    let executor = &CONFIG.get().unwrap().shell_executor;
    debug!("Shell run: {}", command);

    let child = 
    if let Some(executor) = executor {
        // 对于自定义的执行器
        debug!("Shell executor: {}", executor);

        let mut it = executor.as_str().split_ascii_whitespace();
        Command::new(it.next()
            .expect("Error when parsing first executor argument"))
            .args(it)
            .arg(command)
            .spawn()

    } else if cfg!(windows) {
        Command::new("cmd")
            .arg("/c")
            .arg(command)
            .spawn()

    } else if cfg!(unix) {
        Command::new("bash")
            .arg("-c")
            .arg(command)
            .spawn()

    } else {
        error!("shell_runner: Neither Supported Executor OS nor shell_executor specified!");
        return None;

    }.expect("shell_runner: spawn shell failed!");
    
    debug!("Child process #{} started.", child.id());
    Some(tokio::spawn(wait_child_output(child)))

}


async fn wait_child_output(child: Child) {
    let pid = child.id();
    let o = child.wait_with_output().expect("Error waiting for child process");

    info!(
        "Child process #{} exited: Output(ExitStatus=\"{}\", stdout=\"{}\", stderr=\"{}\")",
        pid, o.status,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}
