//! This shows how to write text to serial0.
//! You can see the output with `espflash` if you provide the `--monitor` option

#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{iso_8859_2::FONT_5X7, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::{Dimensions, Point},
    text::{Alignment, Text},
    Drawable,
};
use esp32c3_hal::{
    clock::ClockControl, gpio::IO, i2c::I2C, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc,
    Serial,
};
use esp_backtrace as _;
use esp_println::println;
use nb::block;
use riscv_rt::entry;
use ssd1306::{prelude::DisplayConfig, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
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

    timer0.start(1u64.secs());

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio1,
        io.pins.gpio2,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    println!("hell1o1");
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .build();
    timer0.start(1u64.secs());
    println!("hello");

    loop {
        println!("hello2222");
        Text::with_alignment("Hello world", Point::zero(), text_style, Alignment::Left)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        block!(timer0.wait()).unwrap();
        Text::with_alignment(
            "Hello world2",
            display.bounding_box().center(),
            text_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        block!(timer0.wait()).unwrap();
        display.clear();
    }
}
