pub mod client;
pub mod shell;
pub mod config;
use tokio::{signal, sync::broadcast, task::JoinSet};
use log::{debug, info, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    // 检测配置文件是否合法
    let config;
    match config::from_file(config::CONFIG_PATH) {
        Ok(_c) => config = _c,
        Err(err) => {
            // 如果不合法就创建一个新的
            error!("Error when reading config file! {}", err);
            info!("Created new demo config file in place.");
            config::save_demo().unwrap();

            return;
        }
    }
    

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
    
    
    // 创建用于gracefully kill thread的广播
    let (tx, mut rx1) = broadcast::channel::<bool>(2);
    let tx1 = tx.clone();

    // 创建 Join Handler Set 并创建线程
    let mut join_set = JoinSet::new();
    join_set.spawn(async move {
        let mut retry_time = 0;
        loop {
            tokio::select! {
                // 不要在这里直接使用模式匹配Some(js)，会卡死
                js_res = device_mqtt.poll() => {
                    // 网络正常，读取到数据包
                    if let Ok(js_ok) = js_res { 
                        // 读取到 json，开始处理
                        if let Some(js) = js_ok{
                            if let Some(cmd) = shell::extract_command(&js) {
                                shell::shell_runner(cmd);
                            }
                        }
                        // 重置retry次数
                        retry_time = 0;

                    } else if let Err(e) = js_res {
                        
                        // 超过重试次数就自行退出
                        if config.retry_times >= 0 && retry_time > config.retry_times {
                            error!("Polling Failed out of max retry times.");
                            tx1.send(true).expect("Send kill signal error inside theard.");
                        }

                        retry_time += 1;
                        error!("!ERROR Polling! Tried times:{}, {:?}", retry_time, e);
                    }
                },
                _ = rx1.recv() => {device_mqtt.disconnect().await; break;}
            }
        }
    });
    
    // 等待ctrl_c信号
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Ctrl-C detected!");
            tx.send(true).expect("Send kill signal error.");
            debug!("Kill signal Sent.");
        },
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        },
    }

    // 发送关闭信号给应用所在的任务，然后等待
    debug!("Joining all threads in join_set...");
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(()) => debug!("Thread exited."),
            Err(err) => error!("Thread exited with error: {:?}", err),
        }
        
    }


}
