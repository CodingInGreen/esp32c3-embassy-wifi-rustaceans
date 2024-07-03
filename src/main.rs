//! Access point with station
//!
//! Set SSID and PASSWORD env variable before running this example.
//!
//! Gets an ip address via DHCP, creates an open access-point with SSID `esp-wifi`
//! You can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
//! Open http://192.168.2.1:8080/ in your browser - the example will perform an HTTP get request to some "random" server
//!
//! On Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`

//% FEATURES: esp-wifi esp-wifi/wifi-default esp-wifi/wifi esp-wifi/utils
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
};
use esp_println::{print, println};
use esp_wifi::{
    current_millis,
    initialize,
    wifi::{
        utils::{create_ap_sta_network_interface, ApStaInterface},
        AccessPointConfiguration,
        ClientConfiguration,
        Configuration,
    },
    wifi_interface::WifiStack,
    EspWifiInitFor,
};
use smoltcp::iface::SocketStorage;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    #[cfg(target_arch = "xtensa")]
    let timer = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    #[cfg(target_arch = "riscv32")]
    let timer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;

    let mut ap_socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut sta_socket_set_entries: [SocketStorage; 3] = Default::default();

    let ApStaInterface {
        ap_interface,
        sta_interface,
        ap_device,
        sta_device,
        mut controller,
        ap_socket_set,
        sta_socket_set,
    } = create_ap_sta_network_interface(
        &init,
        wifi,
        &mut ap_socket_set_entries,
        &mut sta_socket_set_entries,
    )
    .unwrap();

    let mut wifi_ap_stack = WifiStack::new(ap_interface, ap_device, ap_socket_set, current_millis);
    let wifi_sta_stack = WifiStack::new(sta_interface, sta_device, sta_socket_set, current_millis);

    let client_config = Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.try_into().unwrap(),
            password: PASSWORD.try_into().unwrap(),
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "esp-wifi".try_into().unwrap(),
            ..Default::default()
        },
    );
    let res = controller.set_configuration(&client_config);
    println!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    println!("is wifi started: {:?}", controller.is_started());

    println!("{:?}", controller.get_capabilities());

    wifi_ap_stack
        .set_iface_configuration(&esp_wifi::wifi::ipv4::Configuration::Client(
            esp_wifi::wifi::ipv4::ClientConfiguration::Fixed(
                esp_wifi::wifi::ipv4::ClientSettings {
                    ip: esp_wifi::wifi::ipv4::Ipv4Addr::from(parse_ip("192.168.2.1")),
                    subnet: esp_wifi::wifi::ipv4::Subnet {
                        gateway: esp_wifi::wifi::ipv4::Ipv4Addr::from(parse_ip("192.168.2.1")),
                        mask: esp_wifi::wifi::ipv4::Mask(24),
                    },
                    dns: None,
                    secondary_dns: None,
                },
            ),
        ))
        .unwrap();

    println!("wifi_connect {:?}", controller.connect());

    // wait for STA getting an ip address
    println!("Wait to get an ip address");
    loop {
        wifi_sta_stack.work();

        if wifi_sta_stack.is_iface_up() {
            println!("got ip {:?}", wifi_sta_stack.get_ip_info());
            break;
        }
    }

    println!("The program has ended....");

    loop {

    }
}

fn parse_ip(ip: &str) -> [u8; 4] {
    let mut result = [0u8; 4];
    for (idx, octet) in ip.split(".").into_iter().enumerate() {
        result[idx] = u8::from_str_radix(octet, 10).unwrap();
    }
    result
}