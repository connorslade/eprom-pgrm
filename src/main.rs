use std::{fs, thread, time::Duration};

use ftdi::Interface;
use ftdi_embedded_hal::{
    FtHal,
    eh0::digital::v2::{InputPin, OutputPin, PinState},
};

fn main() {
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(Interface::A)
        .open()
        .unwrap();
    let hal = FtHal::init_default(device).unwrap();

    let data = [
        hal.ci0().unwrap(),
        hal.ci1().unwrap(),
        hal.ci2().unwrap(),
        hal.ci3().unwrap(),
        hal.ci4().unwrap(),
        hal.ci5().unwrap(),
        hal.ci6().unwrap(),
        hal.ci7().unwrap(),
    ];

    let mut address = [
        hal.ad0().unwrap(),
        hal.ad1().unwrap(),
        hal.ad2().unwrap(),
        hal.ad3().unwrap(),
        hal.ad4().unwrap(),
        hal.ad5().unwrap(),
        hal.ad6().unwrap(), // SR
        hal.ad7().unwrap(), // SR
    ];

    for j in 0..10 {
        let mut rom = Vec::new();
        for i in 0_u16..=0x1FFF {
            set_address(&mut address, i.checked_sub(1), i);
            thread::sleep(Duration::from_millis(5));
            rom.push(read_word(&data));
            // print!("{:0<2X} ", read_word(&data));

            // println!("{i:0>13b} {:0>2X} ", read_word(&data));
            // stdin().read_line(&mut String::new()).unwrap();
        }

        fs::write(format!("dumps/dump_{j}.bin"), rom).unwrap();
    }
}

// sets the  13 address pins
fn set_address(pins: &mut [impl OutputPin; 8], prev: Option<u16>, value: u16) {
    let mut working = value;

    // [6] -> Shift Data
    // [7] -> Shift Clock

    // set first five pins
    for pin in pins.iter_mut().take(6) {
        let state = [PinState::Low, PinState::High][(working & 0b1) as usize];
        working >>= 1;
        unsafe { pin.set_state(state).unwrap_unchecked() };
    }

    const MASK: u16 = 0b1111111000000;
    if let Some(prev) = prev
        && value & MASK == prev & MASK
    {
        return;
    }

    // println!("SETTING SR: {:0>7b}", value >> 6);
    // Update high bits with SR. Clock on rising edge.
    for _ in 0..7 {
        let state = [PinState::Low, PinState::High][(working & 0x40 != 0) as usize];
        working <<= 1;

        // println!("SHIFTING IN: {:?}", state);
        unsafe { pins[7].set_low().unwrap_unchecked() };
        thread::sleep(Duration::from_millis(5));
        unsafe { pins[6].set_state(state).unwrap_unchecked() };
        thread::sleep(Duration::from_millis(5));
        unsafe { pins[7].set_high().unwrap_unchecked() };
    }
}

fn read_word(pins: &[impl InputPin; 8]) -> u8 {
    let mut out = 0;

    for pin in pins.iter().rev() {
        out <<= 1;
        out |= unsafe { pin.is_high().unwrap_unchecked() } as u8;
    }

    out
}
