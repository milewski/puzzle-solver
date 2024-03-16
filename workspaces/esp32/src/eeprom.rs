use esp_idf_hal::delay::{BLOCK, FreeRtos};
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::i2c::{I2c, I2cDriver};
use esp_idf_hal::i2c::config::Config;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::FromValueType;

pub struct Eeprom<'d, const BLOCK_SIZE: usize, const CAPACITY: usize> {
    address: u8,
    driver: I2cDriver<'d>,
}

impl<'d, const BLOCK_SIZE: usize, const CAPACITY: usize> Eeprom<'d, BLOCK_SIZE, CAPACITY> {
    pub fn new<I2C: I2c>(
        i2c: impl Peripheral<P=I2C> + 'd,
        sda: impl Peripheral<P=impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P=impl InputPin + OutputPin> + 'd,
    ) -> Self {
        let config = &Config::default().baudrate(1000.kHz().into());
        let mut driver: I2cDriver = I2cDriver::new(i2c, sda, scl, &config).unwrap();

        Self {
            address: 0b1010_000,
            driver,
        }
    }

    pub fn read_offset(&mut self, offset: u16) -> Vec<u8> {
        let mut response: Vec<u8> = vec![];

        let index = offset.to_le_bytes();
        let mut buffer = [0; BLOCK_SIZE];
        let mut attempts = 0;

        loop {
            if let Ok(_) = self.driver.write_read(self.address, &index, &mut buffer, BLOCK) {
                response.extend_from_slice(buffer.as_slice());

                break;
            }

            if attempts > 10 {
                panic!("Could not read all bytes...");
            }

            attempts += 1;
            FreeRtos::delay_ms(1);
        }

        response
    }

    pub fn write_offset(&mut self, offset: u16, data: &[u8; BLOCK_SIZE]) {
        let mut command: Vec<u8> = offset.to_le_bytes().to_vec();

        command.extend_from_slice(data);

        let mut attempts = 0;

        loop {
            match self.driver.write(self.address, &command.as_slice(), BLOCK) {
                Ok(_) => break,
                Err(_) if attempts < 10 => {
                    attempts += 1;
                    FreeRtos::delay_ms(1);
                }
                Err(error) => panic!("{:?}", error)
            }
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        assert_eq!(data.len() % BLOCK_SIZE, 0, "data should be multiple of {}", BLOCK_SIZE);

        data.chunks(BLOCK_SIZE).enumerate().for_each(|(index, data)| {
            let mut index: Vec<u8> = (index as u16).to_le_bytes().to_vec();

            index.extend_from_slice(data);

            let mut attempts = 0;

            loop {
                match self.driver.write(self.address, index.as_slice(), BLOCK) {
                    Ok(_) => break,
                    Err(_) if attempts < 10 => {
                        attempts += 1;
                        FreeRtos::delay_ms(1);
                    }
                    Err(error) => panic!("{:?}", error)
                }
            }
        });
    }

    pub fn read_all(&mut self) -> Vec<u8> {
        let mut response: Vec<u8> = vec![];

        for block in 0..BLOCK_SIZE {
            response.extend_from_slice(self.read_offset(block as u16).as_slice());
        }

        response
    }

    // pub fn test(&mut self) {
    //     let mut data: Vec<u8> = vec![];
    //
    //     for _ in 1..=BLOCK_SIZE {
    //         data.extend_from_slice(&random_buffer::<BLOCK_SIZE>());
    //     }
    //
    //     self.write(data.as_slice());
    //
    //     let result = self.read_all();
    //     self.reset();
    //
    //     assert_eq!(result, data, "equals");
    // }

    fn reset(&mut self) {
        self.write(&[0; 64 * 128]);
    }
}
