use display_interface::DisplayError;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::Drawable;
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::{DisplayConfig, DisplayRotation, DisplaySize128x64, I2CInterface};
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::sync::{Arc, Mutex};

pub struct DisplayService<'d> {
    device: Ssd1306<
        I2CInterface<I2cDriver<'d>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
}

impl<'d> DisplayService<'d> {
    pub fn new<I2C: I2c>(
        i2c: impl Peripheral<P = I2C> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Arc<Mutex<Self>> {
        let config = I2cConfig::new();
        let driver = I2cDriver::new(i2c, sda, scl, &config).unwrap();

        let interface = I2CDisplayInterface::new(driver);

        let device = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);

        let mut device = device.into_buffered_graphics_mode();

        device.init().unwrap();

        Arc::new(Mutex::new(Self { device }))
    }

    pub fn draw(&mut self, drawable: impl Drawable<Color = BinaryColor>) {
        drawable.draw(&mut self.device).unwrap();
    }

    pub fn flush(&mut self) {
        self.device.flush().unwrap();
    }

    pub fn clear_buffer(&mut self) {
        self.device.clear_buffer()
    }
}
