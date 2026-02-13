# Step 1: Hello, world!

If you followed the instructions in [`README.md`](../README.md), you should have compiled your first firmware that does nothing else than send a "Hello world!" message to your serial terminal.

This file is almost 1:1 the sample code generated for us by `esp-generate`, just with a few things removed for the sake of simplicity.

Let's go through the source code, line by line.

# First part: Attributes, or compiler instructions

We start our file with a few lines that seem syntactically obscure and scary. These are **attributes** in Rust terms: instructions to the compiler.

```rust
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
```

In order, we tell the compiler the following:
* `#![no_std]` specifies that we do not want to include Rust's standard library in this binary. We are compiling for an embedded platform, and the standard library is fairly sizeable and would take a long time to compile.
* `#![no_main]` specifies that we do not want to use the "normal" entry point for the program: this would require the standard library (for complex reasons), and it simply isn't applicable for us due to reasons we'll explain later.
* The two `#![deny(clippy::[...])]` attributes explicitly tell Clippy, Rust's standard linter, that we *never* want to use certain things. `mem_forget` has an explanation for why, and `large_stack_frames` is forbidden due to the constrained RAM we are working with.

> [!TIP]
> Yes, Clippy is probably named after the notoriously annoying Microsoft Office assistant. Unlike its namesake, Rust's Clippy is actually extremely useful and is often your best friend when writing Rust.

# Second part: Imports

This part should not be surprising to you if you are an experienced programmer: `use` is Rust's way of importing items into our current namespace.

```rust
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
```

One noteworthy line is the last one: importing a module can have side effects, and by importing `esp_println` as `_` we are basically giving a strong signal that we are **only** interested in the side effects – by convention, all names starting with an underscore in Rust imply that they are unused. In this case, `esp_println` provides basic line printing implementation for the rest of our program.

> [!TIP]
> The message serialiser we are using is called [defmt](https://defmt.ferrous-systems.com/). It uses an opaque binary protocol for data transfer which is why you only see unreadable binary if you try to use a standard serial monitor. This is governed by a library feature called `defmt-espflash` which you can find in `Cargo.toml` – but that is outside the scope of this tutorial.

## Third part: Some more plumbing

Rust does not have exceptions like many programming languages do: rather, Rust `panic`s. ("Mood", as they say.) Because we are not using the standard library, we need to manually specify what we do on a `panic`.

```rust
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

By annotating this function with `#[panic_handler]`, we basically tell Rust that we want to do nothing and just loop forever on a panic, essentially just halting the program. (Indeed, "mood".) The `!` return type means the function never returns.

> [!TIP]
> Notice the difference between the earlier attributes with an exclamation mark, and this one without it? Attributes with an exclamation mark are called internal attributes and affect whatever they are in – in our case, the whole file and thus the binary. Attributes without it are called external attributes and affect whatever follow them, like the `panic()` function in this case.

The following two non-comment lines are just some additional plumbing:

```rust
extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();
```

The first one is fairly interesting: we tell the Rust compiler that while we do not have the standard library – including its normal memory allocator – we _do_ have an external heap allocator. In our case it is provided by the [esp_alloc crate](https://docs.espressif.com/projects/rust/esp-alloc/0.8.0/esp_alloc/) which you do not really need to care about.

What this basically means is that we can use data types that require heap allocation such as `Vec` (think "list") or `String` which will be **extremely** useful later during the tutorial. Without them we would need to reserve static buffers for any complex datatypes which can be awkward.

> [!TIP]
> Distributable Rust libraries or other compilation units are called "crates" by convention to match the "cargo" name for the package manager.

## Fourth part: Finally, actual code?

Before our main function we see two things:

```rust
#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
```

First, we allow the `large_stack_frames` Clippy check that we forbid before, but we do it only for the `main` function. The reason is explained inline: `main` is the place where we want to allocate large buffers if we do it anywhere.

The second attribute is more interesting: remember how in the beginning of the file we told the Rust compiler we don't have a regular `main` function? Here we mark this function as the entry point for the `esp_rtos` crate. There is a lot of low-level black magic happening behind the scenes, but this is just us telling our whole software stack that hey, our real `main` function is here.

> [!TIP]
> Hey, notice how it's actually `async`? Yes, we are doing pure asynchronous programming on an embedded system. Remember how we called the software stack "almost cheating"?

## Fifth part: Oh, still initialising things

Unfortunately we don't _quite_ get to the good stuff here yet: if you think about it, it makes sense that we need to initialise some hardware and software in our actual `main` function.

```rust
// Initialise the hardware with default options
let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);

// Reclaim heap from the first-stage bootloader
esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

// Start the real-time operating system using the default timer group
let timg0 = TimerGroup::new(peripherals.TIMG0);
esp_rtos::start(timg0.timer0);
```

The comments are rather self-explanatory, but one thing cannot be emphasised enough: `esp_hal` is doing **enormously much** heavy lifting for us here. If you have ever worked with microcontrollers where you need to tweak CPU registers to get specific features on? Disable hardware watchdogs so your microcontroller does not boot every five seconds? This is all done by `esp_hal` under the hood.

> [!NOTE]
> If you encounter any issues running this binary on the badge, it is because of a quirk of the ESP32-S3 chip that we are using: ramping up the CPU clock actually needs to be done in certain steps and it can be unstable otherwise. This is ameliorated in the second part, but based on our testing this **usually** works enough for a hello world example. (That was a "fun" evening of debugging!)

## Sixth part: FINALLY the actual code

```rust
loop {
    info!("Hello world!");
    Timer::after(Duration::from_secs(1)).await;
}
```

After 45 lines of boilerplate and initialisation, we _finally_ have our actual code: write a "Hello world!" message to our equivalent of stdout, which is configured to be the serial interface, and wait one second. Rinse and repeat, forever.

Oh, and because we promised to go line by line?

```rust
}
```

There's your last line.

# Compiling, flashing, and serial monitoring: what is `cargo run` doing?

There are actually three steps to what `cargo run` does. (Spoiler alert: they are in the heading.)

First, as `cargo run` means running the binary, it implies building the binary first. Nothing too fancy here: just a compiler doing compiler things, a linker doing linker things, and so on. After it is finished, you see something like the following:

```
Finished `dev` profile [optimized + debuginfo] target(s) in 0.68s
Running `espflash flash --monitor --chip esp32s3 --log-format defmt target/xtensa-esp32s3-none-elf/debug/step_01_hello_world`
```

Second, we have configured `cargo run` to actually mean the following command: `espflash flash --monitor --chip esp32s3 --log-format defmt` which you see in the log. The `espflash` utility is exactly what you would expect: a tool for flashing ESP series chips. So as the second part, it flashes the compiled binary to the badge. After this is finished, you see something like the following:

```
[2026-02-13T11:30:01Z INFO ] Flashing has completed!
```

Third, as we pass the `--monitor` argument to `espflash`, it automatically starts monitoring the serial interface. You can first see the ESP32 bootloader starting up:

```
I (43) boot: ESP-IDF v5.4.1-426-g3ad36321ea 2nd stage bootloader
[...a couple dozen lines removed...]
I (154) boot: Disabling RNG early entropy source...
```

> [!TIP]
> Wait, **2nd stage**? There is a minimal first stage bootloader that enable features like flash encryption and secure boot – you can read more about this in [Espressif's API guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/bootloader.html).

Finally, you see our own application output:

```
[INFO ] Hello world! (step_01_hello_world src/bin/step_01_hello_world.rs:47)
```

> [!TIP]
> We have a lot of binaries in `src/bin/` – how does `cargo` know which to run? The answer is that we either tell it manually with the `--bin` argument, or it follows the default as given in `Cargo.toml`. (Line 6 in our case.)

## Running the code

You already should have done so, but you can run the code with `cargo run` (or `cargo run --bin step_01_hello_world`).

# Onto the second step of our tutorial

In this step of the tutorial, we basically just introduced ready-made crates (libraries, frameworks) and connected them together like a jigsaw puzzle. However, we are not actually doing **anything related to this badge in particular**: this code would run on (almost) any ESP32-S3 hardware, from devkits to smart plugs!

In the second step of the tutorial, we start using the most important part of any electronics project: the LEDs.

[**Go to the second step!**](002-leds.md)

# Suggested learning steps

* None, just proceed with the tutorial for now.
