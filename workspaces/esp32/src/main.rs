use std::ops::{Add, AddAssign, Mul, MulAssign, Rem, Sub};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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
use shared::puzzle::{Event, PuzzleManager};
use eeprom::Eeprom;

use crate::display::DisplayService;
use crate::utilities::Randomizer;

mod display;
mod utilities;
mod eeprom;

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();

    let mut eeprom: Arc<Mutex<Eeprom<32, 128>>> = Arc::new(Mutex::new(Eeprom::new(
        peripherals.i2c1,
        peripherals.pins.gpio7,
        peripherals.pins.gpio6,
    )));

    let display = DisplayService::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    );

    let mut puzzle = PuzzleManager::new(Randomizer {})?;
    let puzzle_number = 66;

    let display_clone = display.clone();
    let mut eeprom_clone = eeprom.clone();

    puzzle.start_embedded(puzzle_number, FreeRtos, move |event| {
        match event {
            Event::SolutionFound(solution) => {

                {
                    let mut eeprom_answer = solution.to_private_key();

                    println!("{:?}", eeprom_answer);

                    eeprom_clone.lock().unwrap().write_offset(
                        puzzle_number as u16,
                        eeprom_answer.as_slice().try_into().unwrap(),
                    );
                }

                {
                    let answer_string = solution.to_hex();

                    let text = Text::with_alignment(
                        answer_string.as_str(),
                        Point::new(60, 32),
                        MonoTextStyle::new(&PROFONT_7_POINT, BinaryColor::On),
                        embedded_graphics::text::Alignment::Center,
                    );

                    let mut display = display_clone.lock().unwrap();

                    display.clear_buffer();
                    display.draw(text);
                    display.flush();
                }
            }
            Event::SolutionNotFound => {}
        }
    })?;

    {
        let value = eeprom.lock().unwrap().read_offset(puzzle_number as u16);
        let value = BigUint::from_bytes_be(value.as_slice());

        println!("Current answer stored in EEPROM: {:?}", value.to_str_radix(16));
    }

    {
        let text = format!("Working on puzzle: {}", puzzle_number);
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

    // {
    //     let mut eeprom_answer = answer.to_bytes_le();
    //     eeprom_answer.resize(32, 0);
    //
    //     eeprom.write_offset(
    //         puzzle_number as u16,
    //         eeprom_answer.as_slice().try_into().unwrap(),
    //     );
    // }

    // {
    //     let answer_string = answer.to_str_radix(16);
    //
    //     let text = Text::with_alignment(
    //         answer_string.as_str(),
    //         Point::new(60, 32),
    //         MonoTextStyle::new(&PROFONT_7_POINT, BinaryColor::On),
    //         embedded_graphics::text::Alignment::Center,
    //     );
    //
    //     let mut display = display.lock().unwrap();
    //
    //     display.clear_buffer();
    //     display.draw(text);
    //     display.flush();
    // }

    loop {
        FreeRtos::delay_ms(1000);
    }
}
