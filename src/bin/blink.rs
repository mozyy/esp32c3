//! I2C Display example
//!
//! This example prints some text on an SSD1306-based
//! display (via I2C)
//!
//! The following wiring is assumed:
//! - SDA => GPIO1
//! - SCL => GPIO2

#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use core::{cell::RefMut, time::Duration};

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_hal::timer::CountDown;

use esp32c3_hal::{
    adc::{AdcConfig, Attenuation, ADC, ADC1},
    analog::SarAdcExt,
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
    pac::{Peripherals, I2C0, TIMG0},
    prelude::*,
    timer::{Timer, Timer0, TimerGroup},
    Delay, Rtc,
};
use esp_backtrace as _;
use esp_println::println;
use lcd1602_rs::LCD1602;
use nb::block;
use riscv_rt::entry;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use void::Void;

struct TTime(Timer<Timer0<TIMG0>>);
impl CountDown for TTime {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let time: Self::Time = count.into();
        self.0.start((time.as_nanos() as u64).nanos());
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        self.0.wait()
    }
}

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
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

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_println::println!("hello");

    // Init pins
    let rs = io.pins.gpio1.into_push_pull_output();
    let en = io.pins.gpio0.into_push_pull_output();
    let d4 = io.pins.gpio4.into_push_pull_output();
    let d5 = io.pins.gpio5.into_push_pull_output();
    let d6 = io.pins.gpio6.into_push_pull_output();
    let d7 = io.pins.gpio7.into_push_pull_output();
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, TTime(timer0)).unwrap();

    // timer0.start(1u64.secs());

    loop {
        esp_println::println!("print helloworld");
        lcd.print("xu jia xin!").unwrap();
        lcd.delay(2_000_000_u64).unwrap();
        lcd.clear().unwrap();
        lcd.print("beautiful girl!").unwrap();
        lcd.delay(2_000_000_u64).unwrap();
        lcd.clear().unwrap();
    }
}
