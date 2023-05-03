use anyhow::{bail, Result};
use embedded_svc::{
    http::{client::Client as HttpClient, Status},
    io::Write,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi},
};
use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    peripheral,
    peripherals::Peripherals,
    prelude::*,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::{Configuration as HttpConfiguration, EspHttpConnection},
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use log::{debug, error, info};
use serde_json::json;
use shared_bus::BusManagerSimple;
use shtcx::{shtc3, Measurement, PowerMode};
use std::{net::Ipv4Addr, time::Duration};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    api_key: &'static str,
    #[default("")]
    data_source: &'static str,
    #[default("")]
    database: &'static str,
    #[default("")]
    collection: &'static str,
    #[default("")]
    app_id: &'static str,
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    let bus = BusManagerSimple::new(i2c);
    let mut sht = shtc3(bus.acquire_i2c());
    sht.start_measurement(PowerMode::NormalMode).unwrap();

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    // Create an HTTP client
    let mut client = HttpClient::wrap(EspHttpConnection::new(&HttpConfiguration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // Needed for HTTPS support
        ..Default::default()
    })?);

    loop {
        // Get the measurement result
        let measurement = sht.get_measurement_result().unwrap();
        info!(
            "Temp: {} Humidity: {}",
            measurement.temperature.as_degrees_celsius(),
            measurement.humidity.as_percent()
        );
        // Send the measurement result to MongoDB
        post_request(&mut client, measurement, &app_config)?;
        sht.start_measurement(PowerMode::NormalMode).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn post_request(
    client: &mut HttpClient<EspHttpConnection>,
    measurement: Measurement,
    app_config: &Config,
) -> Result<()> {
    // Create the request payload
    let payload = json!(
    {
        "dataSource": app_config.data_source,
        "database" : app_config.database,
        "collection" : app_config.collection,
        "document" : { "temperature": measurement.temperature.as_degrees_celsius(),
                        "humidity": measurement.humidity.as_percent() }
    }
    );
    debug!("Payload: {} \n", payload);
    let content_length_header = format!("{}", payload.to_string().len());
    let headers = [
        ("Content-Type", "application/json"),
        ("api-key", app_config.api_key),
        ("content-length", &*content_length_header),
    ];
    let url = format!(
        "https://data.mongodb-api.com/app/{}/endpoint/data/v1/action/insertOne",
        app_config.app_id
    );
    let mut request = client.post(&url, &headers)?;
    info!("Request sent!");
    request.write_all(payload.to_string().as_bytes())?;
    request.flush()?;
    info!("Pyaload sent!");

    let mut response = request.submit()?;
    let status = response.status();
    debug!("Response status: {}\n", status);
    let (_headers, mut body) = response.split();
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
    debug!("Read {} bytes", bytes_read);
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => debug!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {}", e),
    };

    // Drain the remaining response bytes
    while body.read(&mut buf)? > 0 {}

    Ok(())
}

fn wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        password: pass.into(),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    wifi.start()?;

    info!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    info!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi Connected - DHCP info: {:?}", ip_info);

    Ok(wifi)
}
