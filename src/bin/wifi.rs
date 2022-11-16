//! This shows how to write text to serial0.
//! You can see the output with `espflash` if you provide the `--monitor` option

#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]
// #![feature(default_alloc_error_handler)]

// #[global_allocator]
// static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use embedded_svc::{
    io::{Read, Write},
    wifi::{
        AccessPointInfo, AuthMethod, ClientConfiguration, ClientConnectionStatus, ClientIpStatus,
        ClientStatus, Configuration, Status, Wifi,
    },
};
use esp32c3_hal::{
    clock::ClockControl, pac::Peripherals, prelude::*, systimer::SystemTimer, timer::TimerGroup,
    Rtc, Serial,
};
use esp_backtrace as _;
use esp_println::{logger::init_logger, print, println};
use esp_wifi::{
    ble::controller::BleConnector,
    create_network_stack_storage, current_millis, initialize, network_stack_storage,
    wifi::utils::create_network_interface,
    wifi_interface::{timestamp, Network, WifiError},
};
use nb::block;
use riscv_rt::entry;
use smoltcp::wire::Ipv4Address;

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    // const HEAP_SIZE: usize = 65535;
    // static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    // unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }
    esp_wifi::init_heap();

    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let mut serial0 = Serial::new(peripherals.UART0);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let mut storage = create_network_stack_storage!(3, 8, 1);
    let ethernet = create_network_interface(network_stack_storage!(storage));
    let mut wifi_interface = esp_wifi::wifi_interface::Wifi::new(ethernet);

    {
        let syst = SystemTimer::new(peripherals.SYSTIMER);
        initialize(syst.alarm0, peripherals.RNG, &clocks).unwrap();
    }

    println!("{:?}", wifi_interface.get_status());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
        wifi_interface.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("Call wifi_connect");
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: "x".into(),
        password: "PASSWORD".into(),
        // auth_method: AuthMethod::None,
        ..Default::default()
    });
    let res = wifi_interface.set_configuration(&client_config);
    println!("wifi_connect returned {:?}", res);

    println!("{:?}", wifi_interface.get_capabilities());
    println!("{:?}", wifi_interface.get_status());

    // wait to get connected
    println!("Wait to get connected");
    loop {
        if let Status(ClientStatus::Started(_), _) = wifi_interface.get_status() {
            break;
        }
    }
    println!("{:?}", wifi_interface.get_status());

    // wait for getting an ip address
    println!("Wait to get an ip address");
    loop {
        wifi_interface.poll_dhcp().unwrap();

        wifi_interface
            .network_interface()
            .poll(timestamp())
            .unwrap();

        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(config))),
            _,
        ) = wifi_interface.get_status()
        {
            println!("got ip {:?}", config);
            break;
        }
    }

    let connector = BleConnector {};

    println!("started advertising");

    println!("Start busy loop on main");

    let mut network = Network::new(wifi_interface, current_millis);
    let mut socket = network.get_socket();

    loop {
        println!("Making HTTP request");
        socket.work();

        socket
            .open(Ipv4Address::new(142, 250, 185, 115), 80)
            .unwrap();

        socket
            .write(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
            .unwrap();
        socket.flush().unwrap();

        let wait_end = current_millis() + 2 * 1000;
        loop {
            let mut buffer = [0u8; 512];
            if let Ok(len) = socket.read(&mut buffer) {
                let to_print = unsafe { core::str::from_utf8_unchecked(&buffer[..len]) };
                print!("{}", to_print);
            } else {
                break;
            }

            if current_millis() > wait_end {
                println!("Timeout");
                break;
            }
        }
        println!();

        socket.disconnect();

        let wait_end = current_millis() + 5 * 1000;
        while current_millis() < wait_end {
            socket.work();
        }
    }
}
