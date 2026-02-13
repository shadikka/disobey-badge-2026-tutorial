#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::{Spawner, task};
use embassy_time::{Duration, Timer};
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

use disobey2026badge::*;
use palette::{encoding::Srgb, rgb::Rgb};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Set the palette for our LEDs
const RAINBOW: [Rgb<Srgb, u8>; 6] = [
    Rgb::new(80, 0, 0),
    Rgb::new(80, 80, 0),
    Rgb::new(0, 80, 0),
    Rgb::new(0, 80, 80),
    Rgb::new(0, 0, 80),
    Rgb::new(80, 0, 80),
];

#[task]
async fn led_task(leds: &'static mut Leds<'static>) {
    let mut rainbow_iter = RAINBOW.iter().cycle();
    loop {
        let color = *rainbow_iter.next().unwrap();
        leds.fill(color);
        leds.update().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Initialise the hardware with our badge options!
    let peripherals = disobey2026badge::init();

    // Split the peripherals into more usable resources
    let resources = disobey2026badge::split_resources!(peripherals);

    // Reclaim heap from the first-stage bootloader
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    // Start the real-time operating system using the default timer group
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);
    let leds = mk_static!(Leds<'static>, resources.leds.into());
    info!("Initialised LEDs");

    spawner.must_spawn(led_task(leds));

    loop {
        info!("Main task still alive");
        Timer::after_millis(5000).await;
    }
}
