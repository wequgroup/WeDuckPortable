use reqwest::header;
use serde::Deserialize;
use std::{ops::Deref, time::Duration};
use log::{debug, info, error};

type ReqResult = reqwest::Result<reqwest::Response>;
pub const BASE_URL: &str = "https://api.wequ.net/app";

/// 设备信息结构，从api获取
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct DeviceData {
    pub id: String,
    pub deviceName: String,
    pub deviceStatus: i32,
    pub deviceTopic: String,
    devicePassword: String,
    pub uid: i32,
    pub deviceUrl: Option<String>,
    pub deviceFrom: String,
    pub createTime: String,
    pub updateTime: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    msg: String,
    code: i32,
    pub data: Option<DeviceData>,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug)]
pub struct ApiError {
    code: i32,
    msg: String
}

pub struct MyClient {
    client: reqwest::Client
}

impl Deref for MyClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl MyClient {

    /// 新建一个用于和webapi交互的client来发送ajax请求。
    pub fn new() -> Self {
        // 设置请求头
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, header::HeaderValue::from_static("gzip, identity"));
        headers.insert(header::ACCEPT_CHARSET, header::HeaderValue::from_static("utf-8"));
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json;charset=utf-8"));

        MyClient { 
            client: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .timeout(Duration::new(10, 0))
                .build()
                .expect("WebClient Build Error")
        }
    }

    pub async fn api_get(&self, url: &str) -> ReqResult {
        self.client
            .get(url)
            .send()
            .await
            
    }

    pub async fn api_post<B: Into<reqwest::Body>>(&self, url: &str, body: B) -> ReqResult {
        self.client
            .post(url)
            .body(body)
            .send()
            .await
    }

    /// 通过webapi获取设备信息，返回一个DeviceData
    /// 
    /// # Errors
    /// 
    /// 请求成功，但应用层状态码不为200时，返回ApiError
    /// 请求成功，但没有设备信息时，在msg后加上“未找到设备后”，ApiError
    /// 
    /// # Panics
    /// 
    /// - 发送get请求失败
    /// - 无法将返回的信息反序列化：json => ApiResponse
    pub async fn get_device_data<T>(&self, device_id: T, password: T) -> Result<DeviceData, ApiError>
    where T: AsRef<str>
    {
        let full_url = format!("{}/duck/device/client/{}/{}", BASE_URL, device_id.as_ref(), password.as_ref());
        let res = self.api_get(&full_url)
            .await
            .expect("Failed when fetching device info.")
            .json::<ApiResponse>().await
            .expect("Failed to extract json.");
        
        if res.code != 200 {
            Err(ApiError { code: res.code, msg: res.msg })
        } else { match res.data {
            Some(data) => Ok(data),
            None => Err(ApiError { code: res.code, msg: res.msg + " / 未找到设备" })
        }}
        
    }

}

use rumqttc::{MqttOptions, QoS, AsyncClient, EventLoop, ClientError, Event, Packet};
use serde_json::Value;

pub struct MyMQTT {
    pub client: AsyncClient,
    eventloop: EventLoop,
    topic: String,
}

impl MyMQTT {
    
    /// 读取设备信息，返回专用的MQTT实例。
    /// 在函数内设置了一些预制的MQTT参数。
    pub fn new(device: &DeviceData) -> Self {
        // 获取设备信息
        let device_id = device.id.to_owned();
        let device_pwd = device.devicePassword.to_owned();

        // 设置topic
        let topic = format!("duck/{}", &device_id);

        // 设置MQTT参数
        let mut mqtt_options = MqttOptions::new(&device_id, "mqtt-hw.wequ.net", 1883);
        mqtt_options.set_credentials(&device_id, &device_pwd);
        mqtt_options.set_keep_alive(Duration::from_secs(50));

        // 创建MQTT实例
        let ( client,  eventloop) = AsyncClient::new(mqtt_options, 10);
        MyMQTT { client, eventloop, topic }
    }

    /// 订阅设备频道
    /// 
    /// 订阅实例的 MyMQTT.topic 频道，然后publish一个1
    /// 我也不知道为什么要publish，但是我在抓包的时候看到了这个publish包叠在subscribe包上，就写在这里了。
    pub async fn subscribe(&mut self) -> Result<(), ClientError> {
        debug!("MQTT subscribe topic: {}...", self.topic);
        self.client.try_subscribe(&self.topic, QoS::AtMostOnce)?;
        debug!("MQTT publish test: 1");
        self.client.try_publish(&self.topic, QoS::AtMostOnce, false, "1".as_bytes())
    }

    /// 异步拉取频道信息一次，并且尝试匹配 Publish 包的 Payload。
    /// 
    /// 当接收到一个包时，会在显示在debug级别中。
    /// 如果包长度过短，则在info级别中作为utf8 string输出。
    /// 
    /// # Errors
    /// 
    /// 在poll时网络连接失败，将返回 rumqttc::ConnectionError
    /// 长度过短的payload解析为utf8 string后，返回 Ok(None)
    /// 解析json发生错误，会输出信息到error级别中，但返回 Ok(None)
    pub async fn poll(&mut self) -> Result<Option<Value>, rumqttc::ConnectionError> {

        // raw bytes 2 json
        fn json_from_payload(payload: bytes::Bytes) -> serde_json::Result<Value>{
            let json: Value = serde_json::from_reader(payload.as_ref())?;
            info!("Publish Payload: {:?}", json);
            Ok(json)
        }

        // Event 解包
        match self.eventloop.poll().await {
            Ok(evt) => {
                debug!("Received: {:?}", evt);

                // 如果是 Publish
                if let Event::Incoming(Packet::Publish(pkt)) = evt {
                    debug!("Public: {:?}", pkt);
                    // 尝试parse json

                    // 不解析长度过短 payload 假设不是 json
                    if pkt.payload.len() < 10 {
                        info!("Received payload = \"{}\"", String::from_utf8_lossy(pkt.payload.as_ref()));
                        return Ok(None);
                    }
                    
                    match json_from_payload(pkt.payload) {
                        // 返回 Publish json 数据
                        Ok(json) => Ok(Some(json)),
                        Err(err) => {error!("Error when parse json: {}", err); Ok(None)}
                    }
                
                } else { Ok(None) }
            },
            Err(err) => Err(err)
        }
    }


    /// 断开MQTT频道连接
    pub async fn disconnect(&mut self) {
        debug!("Disconnecting MQTT...");
        self.client.try_disconnect()
            .expect("Error when disconnecting MQTT");
        self.poll().await.ok();
    }
    

}
