#![no_std]
#![no_main]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let pulse_width_us = 1250;
    let pulse_period_us = 20000;

    let mut led_gate = pins.d13.into_output();
    
    led_gate.set_low();
    arduino_hal::delay_ms(3000); // pause to psu to fully enable.
    loop {
        led_gate.set_high();
        arduino_hal::delay_us(pulse_width_us);
        led_gate.set_low();
        arduino_hal::delay_us(pulse_period_us - pulse_width_us);
    }
}
