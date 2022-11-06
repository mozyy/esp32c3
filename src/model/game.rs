extern crate alloc;

use alloc::{
    collections::VecDeque,
    string::{String, ToString},
};
use core::iter::Iterator;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_7X13},
        MonoTextStyle, MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::{Dimensions, Point, Size},
    primitives::{Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Baseline, Text},
    Drawable, Pixel,
};
use esp32c3_hal::{i2c::I2C, pac::I2C0};
use esp_println::println;
use ssd1306::{
    mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x64, Ssd1306,
};

const MAX_SNAKE_LENGTH: usize = 165;

pub type Display =
    Ssd1306<I2CInterface<I2C<I2C0>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

pub enum State {
    Ready,
    Runing,
    Over,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Left,
    Top,
    Right,
    Bottom,
}

pub struct Game<'a> {
    score: i32,
    pub state: State,
    direction: Direction,
    snake_length: i32,
    start: Point,
    width: i32,
    height: i32,
    snake: VecDeque<Point>,
    text_style: MonoTextStyle<'a, BinaryColor>,
    rect_style: PrimitiveStyle<BinaryColor>,
}
impl Default for Game<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Game<'_> {
    pub fn new() -> Self {
        Game {
            score: 0,
            state: State::Ready,
            direction: Direction::Left,
            snake_length: 5,
            start: Point::new(64, 0),
            width: 64,
            height: 64,
            snake: VecDeque::from_iter((10..15).map(|i| Point::new(80, i))),
            text_style: MonoTextStyleBuilder::new()
                .font(&FONT_7X13)
                .text_color(BinaryColor::On)
                .build(),
            rect_style: PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(1)
                // .fill_color(Rgb565::GREEN)
                .build(),
        }
    }
    pub fn set_direction(&mut self, dir: Direction) {
        if match dir {
            Direction::Left => !matches!(self.direction, Direction::Right),
            Direction::Top => !matches!(self.direction, Direction::Bottom),
            Direction::Right => !matches!(self.direction, Direction::Left),
            Direction::Bottom => !matches!(self.direction, Direction::Top),
            _ => true,
        } {
            self.direction = dir;
        }
    }
    pub fn check_closure(&self) -> bool {
        let Point { x: x_min, y: y_min } = self.start;
        let (x_max, y_max) = (x_min + self.width, y_min + self.height);
        // println!(
        //     "point: {:?}, {x_min},{x_max},{y_min},{y_max}",
        //     self.snake.get(0),
        // );
        matches!(self.snake.get(0), Some(Point { x, y }) if *x <= x_min || *x >= x_max || *y <= y_min || *y >= y_max)
        // match self.snake.get(0) {
        //     Some(Point { x, y }) if *x > x_min || *x < x_max || *y > y_min || *y < y_max => true,
        //     _ => false,
        // }
    }
    pub fn next(&mut self) {
        match self.snake.get(0) {
            Some(Point { mut x, mut y }) => {
                match self.direction {
                    Direction::Left => x -= 1,
                    Direction::Top => y -= 1,
                    Direction::Right => x += 1,
                    Direction::Bottom => y += 1,
                };
                let next = Point::new(x, y);
                self.snake.push_front(next);
                self.snake.pop_back();
            }
            None => todo!(),
        };
        // self.direction = dir;
    }
    pub fn draw(&mut self, display: &mut Display) {
        Text::with_baseline(
            (String::from("score:") + &(&self.snake_length - 5).to_string()).as_str(),
            Point::zero(),
            self.text_style,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();
        Rectangle::new(self.start, Size::new(self.width as u32, self.height as u32))
            .into_styled(self.rect_style)
            .draw(display)
            .unwrap();
        self.snake.clone().into_iter().for_each(|node| {
            Pixel(node, BinaryColor::On).draw(display).unwrap();
        });
        match self.state {
            State::Ready => {
                Text::with_baseline(
                    "Ready",
                    Point::new(0, 64),
                    self.text_style,
                    Baseline::Bottom,
                )
                .draw(display)
                .unwrap();
            }
            State::Over => {
                Text::with_baseline("Over", Point::new(0, 64), self.text_style, Baseline::Bottom)
                    .draw(display)
                    .unwrap();
            }
            _ => {}
        }
        display.flush().unwrap();
    }
}
