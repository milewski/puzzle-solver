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
use num_bigint::BigUint;
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

    let mut eeprom: Eeprom<32, 128> = Eeprom::new(
        peripherals.i2c1,
        peripherals.pins.gpio7,
        peripherals.pins.gpio6,
    );

    let display = DisplayService::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    );

    let mut puzzle = Puzzle::number(93);

    {
        let value = eeprom.read_offset(puzzle.number as u16);
        let value = BigUint::from_bytes_le(value.as_slice());

        println!("Current answer stored in EEPROM: {:?}", value.to_str_radix(16));
    }

    {
        let text = format!("Working on puzzle: {}", puzzle.number);
        let text = Text::with_alignment(
            text.as_str(),
            Point::new(60, 32),
            MonoTextStyle::new(&PROFONT_7_POINT, BinaryColor::On),
            embedded_graphics::text::Alignment::Center,
        );

        let mut display = display.lock().unwrap();

        display.clear_buffer();
        display.draw(text);
        display.flush();
    }

    let mut answer = puzzle.start().unwrap();

    {
        let mut eeprom_answer = answer.to_bytes_le();
        eeprom_answer.resize(32, 0);

        eeprom.write_offset(
            puzzle.number as u16,
            eeprom_answer.as_slice().try_into().unwrap(),
        );
    }

    {
        let answer_string = answer.to_str_radix(16);

        let text = Text::with_alignment(
            answer_string.as_str(),
            Point::new(60, 32),
            MonoTextStyle::new(&PROFONT_7_POINT, BinaryColor::On),
            embedded_graphics::text::Alignment::Center,
        );

        let mut display = display.lock().unwrap();

        display.clear_buffer();
        display.draw(text);
        display.flush();
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}
