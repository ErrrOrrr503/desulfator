#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::interrupt::Mutex;
use core::cell::Cell;

use panic_halt as _;

const PRESCALER: u32 = 64;
const TIMER_TOP: u32 = 249;
const MICROS_PER_TIMER_TICK_MUL: u32 = 4;
const MICROS_PER_TIMER_TICK_DIV: u32 = 1;
static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

fn time_timer_init(tc0: &arduino_hal::pac::TC0) {
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| w.bits(TIMER_TOP as u8));
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + 1);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}

fn micros(tc0: &arduino_hal::pac::TC0) -> u32 {
    avr_device::interrupt::free(|cs| {
        let cur_counter = tc0.tcnt0.read().bits();
        MILLIS_COUNTER.borrow(cs).get() * 1000 + cur_counter as u32 * MICROS_PER_TIMER_TICK_MUL / MICROS_PER_TIMER_TICK_DIV
    })
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    time_timer_init(&dp.TC0);
    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    const PULSE_US: u32 = 35;
    const PULSE_PERIOD_US: u32 = 125;
    const POWER_PULSE_US: u32 = 1000;
    const POWER_PULSE_CHARGE_US: u32 = 1000;
    const POWER_PULSE_PERIOD_US: u32 = 8000;

    const END_POWER_CHARGE_US: u32 = POWER_PULSE_US + POWER_PULSE_CHARGE_US;
    const PULSES_AMOUNT: u32 = (POWER_PULSE_PERIOD_US - END_POWER_CHARGE_US) / PULSE_PERIOD_US;
    const END_PULSES: u32 = END_POWER_CHARGE_US + PULSES_AMOUNT * PULSE_PERIOD_US;

    let mut led_gate = pins.d13.into_output();
    
    led_gate.set_low();
    arduino_hal::delay_ms(3000); // pause to psu to fully enable.
    loop {
        let cur_time = micros(&dp.TC0) % POWER_PULSE_PERIOD_US;
        match cur_time {
            0..POWER_PULSE_US => led_gate.set_high(), 
            POWER_PULSE_US..END_POWER_CHARGE_US => led_gate.set_low(),
            END_POWER_CHARGE_US..END_PULSES => {
                let hi_freq_time = (cur_time - END_POWER_CHARGE_US) % PULSE_PERIOD_US;
                match hi_freq_time {
                    0..PULSE_US => led_gate.set_high(), 
                    _ => led_gate.set_low(),
                }
            },
            _ => led_gate.set_low(),
        }
    }
}
