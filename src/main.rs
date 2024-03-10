use std::ops::{Add, AddAssign, Mul, MulAssign, Rem, Sub};
use std::str::FromStr;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::i2c::I2c;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::{FromValueType, Peripherals};
use num_traits::{Num, One, ToPrimitive};
use profont::PROFONT_7_POINT;
use serde::{Deserialize, Serialize};
use eeprom::Eeprom;

use puzzle::Puzzle;

use crate::display::DisplayService;

mod puzzle;
mod display;
mod utilities;
mod eeprom;

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();

    let mut eeprom: Eeprom<64, 128> = Eeprom::new(
        peripherals.i2c1,
        peripherals.pins.gpio7,
        peripherals.pins.gpio6,
    );

    eeprom.test();

    let display = DisplayService::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    );

    let mut puzzle = Puzzle::number(66);
    let result = puzzle.start().unwrap();

    let text = Text::with_alignment(
        result.as_str(),
        Point::new(60, 32),
        MonoTextStyle::new(&PROFONT_7_POINT, BinaryColor::On),
        embedded_graphics::text::Alignment::Center,
    );

    let mut display = display.lock().unwrap();

    display.clear_buffer();
    display.draw(text);
    display.flush();

    loop {
        FreeRtos::delay_ms(1000);
    }
}
