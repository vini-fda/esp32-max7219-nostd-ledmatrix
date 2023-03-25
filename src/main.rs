#![no_std]
#![no_main]

extern crate alloc;
use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use max7219::connectors::PinConnector;
use max7219::MAX7219;
use max7219::DecodeMode;

use esp_max7219_nostd::{prepare_display, show_moving_text_in_loop};

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
        static mut _heap_end: u32;
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        let heap_end = &_heap_end as *const _ as usize;
        assert!(
            heap_end - heap_start > HEAP_SIZE,
            "Not enough available heap memory."
        );
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    println!("Hello world!");

    // Set GPIO2 as an output, and set its state high initially.
    let io = hal::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio2.into_push_pull_output();

    led.set_high().unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = hal::Delay::new(&clocks);

    let din = io.pins.gpio23.into_push_pull_output();
    let cs = io.pins.gpio5.into_push_pull_output();
    let clk = io.pins.gpio18.into_push_pull_output();

    let mut display = MAX7219::from_pins(1, din, cs, clk).unwrap(); // replace "1" with number of displays in chain, if you have more
    prepare_display(&mut display, 1, 0x5);
    show_moving_text_in_loop(
        &mut display, 
        "RUST",
        1, // replace "1" with number of displays in chain, if you have more
        100, 
        2, 
        &mut delay,
    );

    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}
