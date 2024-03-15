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

fn main() -> anyhow::Result<()> {

    let mut puzzle = Puzzle::number(93);

}
