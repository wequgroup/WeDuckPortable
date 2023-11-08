use tokio::{signal, sync::broadcast, task::JoinSet};
pub mod client;
use log::{debug, info, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    // 创建web client对象以调用api
    let web = client::MyClient::new();
    debug!("Fetching device info...");
    let device_info = web.get_device_data("***REMOVED***", "***REMOVED***")
        .await.expect("Failed when fetching device info.");
    info!("Fetched device name: {}", &device_info.deviceName);
    debug!("{:#?}", device_info);
    
    // 创建MQTT通道
    let mut device_mqtt = client::MyMQTT::new(&device_info);
    device_mqtt.subscribe().await.expect("MQTT subscribe error");
    
    
    // 创建用于gracefully kill thread的广播
    let (tx, mut rx1) = broadcast::channel::<bool>(2);

    // 创建 Join Handler Set 并创建线程
    let mut join_set = JoinSet::new();
    join_set.spawn(async move {
        loop {
            tokio::select! {
                _ = device_mqtt.poll() => (),
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
