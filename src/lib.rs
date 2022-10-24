#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use fugit::ExtU32;

struct LedDisplay<PC, PD>
where
    PC: OutputPin,
    PD: OutputPin,
{
    clk: PC,
    dio: PD,
}

impl<PC, PD> LedDisplay<PC, PD>
where
    PC: OutputPin,
    PD: OutputPin,
{
    pub fn new(clk: PC, dio: PD) -> Self {
        Self { clk, dio }
    }

    fn delay<T: CountDown>(timer: &mut T)
    where
        T::Time: From<fugit::MicrosDuration<u32>>,
    {
        timer.start(5.micros());
        timer.wait();
    }

    fn start<T: CountDown>(&mut self, timer: &mut T)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        self.dio.set_low();
        Self::delay(timer);
    }

    fn stop<T: CountDown>(&mut self, timer: &mut T)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        self.dio.set_low();
        Self::delay(timer);
        self.clk.set_high();
        Self::delay(timer);
        self.dio.set_high();
        Self::delay(timer);
    }

    fn write_byte<T: CountDown>(&mut self, timer: &mut T, byte: u8)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        let mut b = byte;
        for i in 0..8 {
            self.clk.set_low();
            Self::delay(timer);
            if (byte & 1) == 1 {
                self.dio.set_high();
            } else {
                self.dio.set_low();
            }
            Self::delay(timer);
            self.clk.set_high();
            Self::delay(timer);
            b >>= 1;
        }
        self.clk.set_low();
        Self::delay(timer);
        self.clk.set_high();
        Self::delay(timer);
        Self::delay(timer);
        self.clk.set_low();
        Self::delay(timer);
    }

    fn write<T: CountDown>(&mut self, timer: &mut T, bytes: &[u8])
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        self.start(timer);
        for byte in bytes {
            self.write_byte(timer, *byte);
        }
        self.stop(timer);
    }

    pub fn set_brightness<T: CountDown>(&mut self, timer: &mut T, brightness: u8)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        let control;

        if brightness == 0 {
            control = 0x80;
        } else {
            if brightness > 8 {
                control = 0x88 | 7;
            } else {
                control = 0x88 | (brightness - 1)
            }
        }
        self.write(timer, &[brightness]);
    }

    fn show_segment<T: CountDown>(&mut self, timer: &mut T, segment: u8, pos: u8)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        if pos > 3 {
            return;
        }

        self.write(timer, &[0x44]); // memory write command

        let buffer: [u8; 2] = [0xc0 + pos, segment];
        self.write(timer, &buffer)
    }

    fn show_digits<T: CountDown>(&mut self, timer: &mut T, nums: &[u8; 4])
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        self.write(timer, &[0x40]); //memory write command (auto increment mode)

        let mut buffer: [u8; 5] = [200; 5];
        buffer[0] = 0xc0; // set address to the first digit
        for i in 0..4 {
            buffer[i + 1] = Self::get_segments(nums[i]);
        }
        self.write(timer, &buffer);
    }

    fn show_number<T: CountDown>(&mut self, timer: &mut T, num: i32)
    where
        <T as CountDown>::Time: From<fugit::MicrosDuration<u32>>,
    {
        let mut digits: [u8; 4] = [200;4];
        let mut n = num;

        for i in 0..4 {
            digits[3-i] = (n % 10).try_into().unwrap();
            n = n / 10;
        }

        self.show_digits(timer, &digits);
    }

    fn get_segments(num: u8) -> u8 {
        match num {
            0 => 0x3f,
            1 => 0x06,
            2 => 0x5b,
            3 => 0x4f,
            4 => 0x66,
            5 => 0x6d,
            6 => 0x7d,
            7 => 0x07,
            8 => 0x7f,
            9 => 0x6f,
            _ => 0x00,
        }
    }
}

enum Error {
    DATA_PIN_ERROR(),
}

#[cfg(test)]
mod tests {
    use super::*;
}
