#![deny(rust_2018_idioms)]
#![feature(iter_intersperse)]

use std::{borrow::Cow, time::Duration};

use anyhow::Result;
use edge_std_nal_async::Stack;
use embedded_nal_async::{Ipv4Addr, SocketAddr};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{peripherals::Peripherals, task::block_on},
    http::{
        server::{Configuration as HttpServerConfiguration, EspHttpServer},
        Method,
    },
    io::Write,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{
        AccessPointConfiguration, AsyncWifi, AuthMethod, Configuration as WifiConfiguration,
        EspWifi, WifiDeviceId,
    },
    ws::FrameType,
};
use log::{info, warn};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let driver = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    let mut wifi = AsyncWifi::wrap(driver, sys_loop, timer_service)?;

    let http_server = block_on(start_http())?;
    block_on(start_dns())?;
    block_on(start_ap(&mut wifi))?;

    Ok(())
}

/// Start the HTTP server.
async fn start_http() -> Result<EspHttpServer<'_>> {
    let mut server = EspHttpServer::new(&HttpServerConfiguration {
        ..Default::default()
    })?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?.write_all(b"Hello")
    })?;
    server.ws_handler("/ws/wifi/connect", |req| {})?;

    Ok(server)
}

/// Start the DNS server.
async fn start_dns() -> Result<()> {
    let stack = Stack::new();
    let mut tx = [0; 1500];
    let mut rx = [0; 1500];

    edge_captive::io::run(
        &stack,
        edge_captive::io::DEFAULT_SOCKET,
        &mut tx,
        &mut rx,
        Ipv4Addr::new(192, 168, 0, 1),
        Duration::from_secs,
    )
    .await?;

    Ok(())
}

/// Start the soft AP with the SSID `ESP <MAC address>` and no auth.
async fn start_ap(wifi: &mut AsyncWifi<EspWifi<'_>>) -> Result<()> {
    let driver = wifi.wifi();
    let ssid = String::with_capacity(21);
    let ssid = driver
        .get_mac(WifiDeviceId::Ap)?
        .into_iter()
        .map(|b| Cow::from(format!("{b:02X}")))
        .intersperse(Cow::from(":"))
        .fold(ssid + "ESP ", |ssid, s| ssid + &s);
    let ssid = heapless::String::<32>::try_from(ssid.as_str()).unwrap();

    wifi.set_configuration(&WifiConfiguration::AccessPoint(AccessPointConfiguration {
        ssid,
        channel: 1,
        secondary_channel: Some(6),
        ..AccessPointConfiguration::default()
    }))?;
    wifi.start().await?;
    info!("AP started");
    wifi.wait_netif_up().await?;
    info!("AP netif up");

    match driver.ap_netif().get_ip_info() {
        Ok(ip_info) => info!("AP IP info: {ip_info:?}"),
        Err(e) => warn!("Failed to get AP IP info: {e}"),
    }

    Ok(())
}
