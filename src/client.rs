use reqwest::header;
use serde::Deserialize;
use std::{ops::Deref, time::Duration};
use log::{debug, info};

type ReqResult = reqwest::Result<reqwest::Response>;
pub const BASE_URL: &str = "https://api.wequ.net/app";

#[allow(dead_code, non_snake_case)]
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

    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, header::HeaderValue::from_static("gzip, identity"));
        headers.insert(header::ACCEPT_CHARSET, header::HeaderValue::from_static("utf-8"));
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json;charset=utf-8"));
        
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::new(10, 0));
        let client = client.build()
            .expect("WebClient Build Error");
        MyClient { client }
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
            None => Err(ApiError { code: res.code, msg: res.msg + " -- No device data." })
        }}
        
    }

}

use rumqttc::{MqttOptions, QoS, AsyncClient, EventLoop, ClientError};

pub struct MyMQTT {
    pub client: AsyncClient,
    eventloop: EventLoop,
    device_id: String,
    device_pwd: String
}

impl MyMQTT {
    
    pub fn new(device: &DeviceData) -> Self {
        let device_id = device.id.to_owned();
        let device_pwd = device.devicePassword.to_owned();
        let mut mqtt_options = MqttOptions::new("mqtt_id_test", "mqtt-hw.wequ.net", 1883);
        mqtt_options.set_credentials(&device_id, &device_pwd);
        let ( client,  eventloop) = AsyncClient::new(mqtt_options, 10);
        MyMQTT { client, eventloop, device_id, device_pwd }
    }

    pub async fn subscribe(&mut self) -> Result<(), ClientError> {
        let channel = format!("duck/{}", self.device_id);
        debug!("MQTT subscribe channel: {}", channel);
        self.client.subscribe(channel, QoS::AtLeastOnce).await
    }

    pub async fn poll(&mut self) {
        match self.eventloop.poll().await {
            Ok(msg) => info!("Received: {:?}", msg),
            Err(err) => info!("!ERROR! Polling: {:?}", err),
        }
    }

    pub async fn disconnect(&mut self) {
        debug!("Disconnecting MQTT...");
        self.client.try_disconnect()
            .expect("Error when disconnecting MQTT")
    }
    

}
