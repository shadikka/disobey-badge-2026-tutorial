# Step 2: Blinkenlights

Like discussed at the end of step 1, nothing we have written so far is specific to our Disobey 2026 badge: it doesn't use any of the specific hardware we have at our disposal, except for standard ESP32-S3 features.

A wise person once said that every electronics project is a LED blinker, some projects just do something else too. In the spirit of that, let's put the WS2812-compatible LEDs in our badge to use!

## New imports

We have some new imports in our code:

```rust
use disobey2026badge::*;
use palette::{encoding::Srgb, rgb::Rgb};
```

The first one is the BSP, or _board support package_ for our badge. It contains all the information about what sort of peripherals connected to which GPIO pin, and a few supporting features (like the CPU speed ramp-up mentioned previously).

The second one is a common Rust crate for handling RGB colours.

> [!TIP]
> If you were to add these yourself, you would need to run the following commands:
> 
> ```rust
> cargo add disobey2026badge
> cargo add palette --no-default-features --features alloc,approx,named,named_from_str,phf,libm
> ```
> 
> The second command looks complicated because the `palette` crate by default depends on the standard library being present – we want just about every feature except the `std` one from it.

## Code changes

Instead of using `esp_hal::Config::default()` and `esp_hal::init(config)`, we now use the hardware initialisation function provided to us by the BSP:

```rust
// Initialise the hardware with our badge options!
let peripherals = disobey2026badge::init();

// Split the peripherals into more usable resources
let resources = disobey2026badge::split_resources!(peripherals);
```

In addition to initialising the "raw" peripherals, we also split them into _resources_ which are higher-level and more practical to use.

## New code

We start off on line 50 by specifying a constant RGB palette:

```rust
// Set the palette for our LEDs
const RAINBOW: [Rgb<Srgb, u8>; 6] = [
    Rgb::new(80, 0, 0),
    Rgb::new(80, 80, 0),
    Rgb::new(0, 80, 0),
    Rgb::new(0, 80, 80),
    Rgb::new(0, 0, 80),
    Rgb::new(80, 0, 80),
];
```

Besides maybe the syntax, there should be no major surprises here. The values are 80 instead of 255 because the LEDs are **quite** bright enough that you don't want them at full blast.

Then we make an _iterator_ for this palette:

```rust
let mut rainbow_iter = RAINBOW.iter().cycle();
```

The normal `.iter()` creates an iterator that runs once and stops returning values after that – we want it to be infinite, so we add `.cycle()` there which will keep on repeating the iterator from the beginning.

We also need to convert the LED strip into an _even_ more useful form:

```rust
let mut leds: disobey2026badge::Leds<'_> = resources.leds.into();
```

> [!TIP]
> Notice the `mut` in both variable declarations? We need to mutate the state of both of these structs so we need to make them mutable. By default everything in Rust is immutable! (The notable exception is [interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html) which we will very briefly cover later.)

Finally, in our main loop we simple get the next colour from the iterator and apply it to our LEDs. Note that the struct we have for the LEDs is buffered: we need to explicitly flush the buffer to the physical device.

```rust
loop {
    let color = *rainbow_iter.next().unwrap();
    leds.fill(color);
    leds.update().await;
    Timer::after(Duration::from_secs(1)).await;
}
```

> [!TIP]
> The `.unwrap()` is generally frowned upon as it `panic`s (crashes the whole program) if used on a value that is empty or an error (most commonly `None` or `Err`). Here we can be certain we always get a value only because the iterator is endless as guaranteed by `.cycle()`.
> 
> In general if you find yourself using `.unwrap()`, think twice!

## Running the code

Run the code with `cargo run --bin step_02_leds` and enjoy the blinkenlights.

# Continuing to the third step

So far we are doing very much one thing – changing our LED colour – despite having a full-blown asynchronous execution framework with bells and whistles. In the third step, we will move this to its own _task_, showcasing how useful Embassy can be.

[Go to the third step](003-tasks.md)

# Suggested learning tasks

* Easy: Change the palette to something that suits your personal frivolities and flamboyances better.
* Medium: Make the palette interpolate from one color to another on a shorter timer.
