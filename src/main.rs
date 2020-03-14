mod io;

use embedded_hal::blocking::{delay::DelayMs, i2c};
use linux_embedded_hal::{Delay, I2cdev};
use std::{thread, time};
use crate::io::ezortd::EzoRtd;

const ADDR_EZO_RTD: u8 = 0x66;

fn main() {
    println!(r" ___         _          ___          __ ");
    println!(r"| _ \_  _ __| |_ _  _  | _ \___ ___ / _|");
    println!(r"|   / || (_-<  _| || | |   / -_) -_)  _|");
    println!(r"|_|_\\_,_/__/\__|\_, | |_|_\___\___|_|  ");
    println!(r"                 |__/                   "); 

    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let delay = Delay{};

    let mut rtd = EzoRtd::new(i2c, delay, ADDR_EZO_RTD);
    loop {
        println!("{:?}", rtd.read());
        thread::sleep(time::Duration::from_secs(1));
    }
}
