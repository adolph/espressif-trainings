use core::str;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
use embedded_svc::{
    http::{
        server::{registry::Registry, Response, ResponseWrite},
        Method,
    },
    io::Write,
};
use esp32_c3_dkc02_bsc as bsc;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    let _wifi = wifi(CONFIG.wifi_ssid, CONFIG.wifi_psk)?;

    let server_config = Configuration::default();
    let mut server = EspHttpServer::new(&server_config)?;
    server.set_inline_handler("/", Method::Get, |request, response| {
        let html = index_html();
        let mut writer = response.into_writer(request)?;
        writer.do_write_all(html.as_bytes())?;
        writer.complete()
    })?;

    let temp_sensor_main = Arc::new(Mutex::new(BoardTempSensor::new_taking_peripherals()));
    let temp_sensor = temp_sensor_main.clone();

    server.set_inline_handler("/temperature", Method::Get, move |request, response| {
        let temp_val = temp_sensor.lock().unwrap().read_owning_peripherals();
        let html = temperature(temp_val);
        let mut writer = response.into_writer(request)?;
        writer.do_write_all(html.as_bytes())?;
        writer.complete()
    })?;

    println!("server awaiting connection");

    loop {
        sleep(Duration::from_millis(1000));
    }
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html() -> String {
    templated("Hello from mcu!")
}

fn temperature(val: f32) -> String {
    templated(format!("chip temperature: {:.2}°C", val))
}
