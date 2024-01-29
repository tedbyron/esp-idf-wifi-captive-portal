#![deny(rust_2018_idioms)]
#![feature(iter_intersperse)]

use std::{borrow::Cow, thread::sleep, time::Duration};

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{peripherals::Peripherals, task::block_on},
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AccessPointConfiguration, AsyncWifi, AuthMethod, Configuration, EspWifi, WifiDeviceId},
};
use heapless::String;
use log::info;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let driver = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    let mut wifi = AsyncWifi::wrap(driver, sys_loop, timer_service)?;

    block_on(start_ap(&mut wifi))?;

    loop {
        sleep(Duration::from_secs(1));
    }
}

/// Starts the soft AP with the SSID: `ESP <MAC address>`.
async fn start_ap(wifi: &mut AsyncWifi<EspWifi<'_>>) -> Result<()> {
    let ssid = wifi
        .wifi()
        .get_mac(WifiDeviceId::Ap)?
        .into_iter()
        .map(|b| Cow::from(format!("{b:02X}")))
        .intersperse(Cow::from(":"))
        .fold("ESP ".to_string(), |ssid, s| ssid + &s);
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: String::<32>::try_from(ssid.as_str()).unwrap(),
        secondary_channel: Some(6),
        auth_method: AuthMethod::None,
        password: String::<64>::new(),
        ..AccessPointConfiguration::default()
    }))?;
    drop(ssid);

    wifi.start().await?;
    info!("AP started");
    wifi.wait_netif_up().await?;
    info!("AP netif up");

    Ok(())
}
