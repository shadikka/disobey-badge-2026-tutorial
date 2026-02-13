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
use embassy_futures::select::select_array;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
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

#[derive(Clone, Copy, defmt::Format)]
enum ButtonPressEvent {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    STICK,
    A,
    B,
    START,
    SELECT,
}

static BUTTON_CHANNEL: PubSubChannel<CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1> = PubSubChannel::new();
type ButtonSubscriber = Subscriber<'static, CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1>;
type ButtonPublisher = Publisher<'static, CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1>;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Set the palette for our LEDs
const PALETTE: [Rgb<Srgb, u8>; 9] = [
    Rgb::new(80, 0, 0),
    Rgb::new(80, 80, 0),
    Rgb::new(0, 80, 0),
    Rgb::new(0, 80, 80),
    Rgb::new(0, 0, 80),
    Rgb::new(80, 0, 80),
    Rgb::new(80, 80, 80),
    Rgb::new(0, 0, 0),
    Rgb::new(120, 60, 30),
];

#[task]
async fn led_task(mut subscriber: ButtonSubscriber, leds: &'static mut Leds<'static>) {
    loop {
        let event = subscriber.next_message_pure().await;
        let color = match event {
            ButtonPressEvent::UP => PALETTE[0],
            ButtonPressEvent::DOWN => PALETTE[1],
            ButtonPressEvent::LEFT => PALETTE[2],
            ButtonPressEvent::RIGHT => PALETTE[3],
            ButtonPressEvent::STICK => PALETTE[4],
            ButtonPressEvent::A => PALETTE[5],
            ButtonPressEvent::B => PALETTE[6],
            ButtonPressEvent::START => PALETTE[7],
            ButtonPressEvent::SELECT => PALETTE[8],
        };
        leds.fill(color);
        leds.update().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[task]
async fn button_task(publisher: ButtonPublisher, buttons: &'static mut Buttons) {
    loop {
        match select_array([
            Buttons::debounce_press(&mut buttons.up),
            Buttons::debounce_press(&mut buttons.down),
            Buttons::debounce_press(&mut buttons.left),
            Buttons::debounce_press(&mut buttons.right),
            Buttons::debounce_press(&mut buttons.stick),
            Buttons::debounce_press(&mut buttons.a),
            Buttons::debounce_press(&mut buttons.b),
            Buttons::debounce_press(&mut buttons.start),
            Buttons::debounce_press(&mut buttons.select),
        ])
        .await
        {
            ((), 0) => publisher.publish(ButtonPressEvent::UP).await,
            ((), 1) => publisher.publish(ButtonPressEvent::DOWN).await,
            ((), 2) => publisher.publish(ButtonPressEvent::LEFT).await,
            ((), 3) => publisher.publish(ButtonPressEvent::RIGHT).await,
            ((), 4) => publisher.publish(ButtonPressEvent::STICK).await,
            ((), 5) => publisher.publish(ButtonPressEvent::A).await,
            ((), 6) => publisher.publish(ButtonPressEvent::B).await,
            ((), 7) => publisher.publish(ButtonPressEvent::START).await,
            ((), 8) => publisher.publish(ButtonPressEvent::SELECT).await,
            _ => unreachable!(),
        }
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
    let buttons = mk_static!(Buttons, resources.buttons.into());
    info!("Initialised LEDs");

    let mut subscriber = BUTTON_CHANNEL.subscriber().unwrap();
    spawner.must_spawn(led_task(BUTTON_CHANNEL.subscriber().unwrap(), leds));
    spawner.must_spawn(button_task(BUTTON_CHANNEL.publisher().unwrap(), buttons));

    loop {
        let message = subscriber.next_message_pure().await;
        info!("Main received message: {:?}", message);
    }
}
