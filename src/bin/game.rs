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

use core::cell::RefMut;

extern crate alloc;
use alloc::{
    borrow::ToOwned,
    fmt::format,
    string::{String, ToString},
};
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};
use esp32c3_hal::{
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
    pac::{Peripherals, I2C0},
    prelude::*,
    timer::TimerGroup,
    Delay, Rtc,
};
use esp_backtrace as _;
use esp_println::println;
use nb::block;
use riscv_rt::entry;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use esp32c3::model::game::{self, Direction, Display, Game, State};

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

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio5,
        io.pins.gpio4,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    // Start timer (5 second interval)
    timer0.start(5u64.secs());

    // Initialize display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display: Display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let mut delay = Delay::new(&clocks);

    let mut game = Game::new();
    let left = io.pins.gpio6.into_pull_up_input();
    let top = io.pins.gpio7.into_pull_up_input();
    let right = io.pins.gpio8.into_pull_up_input();
    let bottom = io.pins.gpio9.into_pull_up_input();

    let mut move_time = 0;

    let get_button = || -> Option<Direction> {
        if left.is_low().unwrap() {
            Some(Direction::Left)
        } else if top.is_low().unwrap() {
            Some(Direction::Top)
        } else if right.is_low().unwrap() {
            Some(Direction::Right)
        } else if bottom.is_low().unwrap() {
            Some(Direction::Bottom)
        } else {
            None
        }
    };
    game.draw(&mut display);

    loop {
        match game.state {
            game::State::Ready => {
                if let Some(dir) = get_button() {
                    game.set_direction(dir);
                    game.state = State::Runing;
                }
            }
            game::State::Runing => {
                move_time += 1;
                let dir = get_button();
                if let Some(dir) = dir {
                    game.set_direction(dir);
                }
                if move_time > 30 {
                    game.next();
                    display.clear();
                    game.draw(&mut display);
                    if game.check_closure() {
                        game.state = State::Over;
                        display.clear();
                        game.draw(&mut display);
                    }
                    move_time = 0;
                }
            }
            game::State::Over => {
                if get_button().is_some() {
                    game = Game::new();
                    display.clear();
                    game.draw(&mut display);
                }
            }
        }

        delay.delay_ms(10u32);

        // delay.delay_ms(1000u32);
        // block!(timer0.wait()).unwrap();
    }
}
