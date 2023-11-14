//! 使用 Rust 写出的神秘鸭客户端
//! 仅支持命令行方式运行
//! 特点：
//! - 运行快：Rust 语言作为编译型语言，运行速度要高于解释型语言。
//! - 省资源：运行内存占用约 3MB，适合在一些小型设备上运行。（但设备可能也没那么小型）
//! - 少依赖：二进制文件直接运行，不需要环境前置，可以便携运行。
//! - 可自定义运行器：更自然和方便的命令运行方式，并异步等待命令结束，返回命令结果。

pub mod client;
pub mod shell;
pub mod config;

use serde_json::Value;
use structopt::StructOpt;
use tokio::{signal, sync::{mpsc, OnceCell}};
use log::{debug, info, error};

// 仅仅在接收函数的时候初始化一次，之后只读取，不修改
pub static CONFIG: OnceCell<config::AppConfig> = OnceCell::const_new();


/// 处理返回的 `Result<Option<Value>, rumqttc::ConnectionError>``
/// 
/// 当链接错误 rumqttc::ConnectionError 时自动重试（< CONFIG.retry_times）
/// 能够成功解析 shell command 就执行
/// 使用全局变量：CONFIG.max_retry_times
fn poll_handler(
    input: Result<Option<Value>, rumqttc::ConnectionError>,
    retried_time: &mut i32,
) {

    match input {
        // 网络正常，读取到数据包
        Ok(js_ok) => { 
            // 读取到 json，开始处理
            if let Some(js) = js_ok{
                if let Some(cmd) = shell::extract_command(&js) {
                    shell::shell_runner(cmd);
                }
            }
            // 重置retry次数
            *retried_time = 0;
    
        },
        Err(e) => {
            // 超过重试次数就自行退出
            let max_retry_times = CONFIG.get().unwrap().max_retry_times;
            if max_retry_times >= 0 && *retried_time >= max_retry_times {
                panic!("Polling Failed out of max retry times.");
            }
    
            *retried_time += 1;
            error!("!ERROR Polling! Tried times:{}, {:?}", retried_time, e);
        }
    }
}

#[tokio::main]
async fn main() {
    // 从命令行读取配置
    match config::AppConfig::from_args_safe() {
        Ok(cfg) => CONFIG.set(cfg).unwrap(),
        Err(err) => {println!("{}", err); return;},
    };
    let config = CONFIG.get().unwrap();

    // 初始化log环境
    if let Some(l) = &config.log_level {
        std::env::set_var("RUST_LOG", l);
    }
    env_logger::init();

    // 创建web client对象以调用api
    let web = client::MyClient::new();
    debug!("Fetching device info...");
    let device_info = web.get_device_data(&config.device_id, &config.device_password)
        .await.expect("Failed when fetching device info.");
    info!("Fetched device name: {}", &device_info.deviceName);
    debug!("{:#?}", device_info);
    
    // 创建MQTT通道
    let mut device_mqtt = client::MyMQTT::new(&device_info);
    device_mqtt.subscribe().await.expect("MQTT subscribe error");
    
    
    // 创建用于gracefully kill thread的广播。mpsc-多发送单接收
    // 首先堵塞通道占满buffer，
    // 如果子线程发送端返回（如通道/接收端被关闭返回错误、缓存没满发送成功），就结束
    let (tx, rx) = mpsc::channel::<()>(1);
    tx.send(()).await.expect("Error when sending first msg in mpsc broadcast channel.");

    // 创建线程组并创建线程
    let mut children = Vec::new();
    children.push(tokio::spawn(async move {
        
        let mut retry_time = 0;
        let tx1 = (&tx).clone();

        loop { tokio::select! {
            // 不要在这里直接使用模式匹配Some(js)，会卡死
            out = device_mqtt.poll() => poll_handler(
                out, &mut retry_time
            ),
            _ = tx1.send(()) => {device_mqtt.disconnect().await; break;}
        }}
    }));

    // 等待ctrl_c信号或所有子线程结束
    tokio::select! {
        _ = futures::future::join_all(children) => {
            debug!("All child threads ended.");
            // 代码结束，rx自动被drop
        }
        _ = signal::ctrl_c() => {
            info!("Ctrl-C Signal, stopping...");
            // 关闭通道的接收端，让所有尝试发送的发送端立刻返回
            drop(rx);
        }
    };

}
