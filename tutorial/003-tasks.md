# Third steps: Tasks and Embassy

Like mentioned at the end of our second step, we are running just a single task despite having the potential to do _so much more_. This is a bit like using an unpowered jackhammer as a chisel: impractical, unwieldy, and just a bit silly.

So let's fix this!

## New imports

```rust
use embassy_executor::{Spawner, task};
```

Here we have added the `task` macro from `embassy_executor` to our scope. This is an attribute you can use to mark an asynchronous function as a task that Embassy, our execution framework, can run.

## Code changes

We have moved our colour palette to outside the `main` function for reasons that will soon become evident.

```rust
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
```

The code here should look largely familiar: it's just the main loop from the previous step but in its own function and the palette moved out of the function.

Two noteworthy things here otherwise:

```rust
#[task]
```

Like discussed above, this marks our `led_task` function as suitable for execution as an Embassy task.

```rust
async fn led_task(leds: &'static mut Leds<'static>)
```

This might look scary, but take a deep breath: we tell the compiler that the only parameter for this function is:

* A reference `&`
* To a static resource – something that is _always_ available to the program, not temporary (`'static`)
* Mutable (`mut`)
* Of the `disobeybadge2026::Leds` type with the lifetime `'static` – we can drop the `disobeybadge2026::` as we have used a wildcard import there.

How do we know it's a static resource that is always there? Read on, in the `main` function we do the following:

```rust
let leds = mk_static!(Leds<'static>, resources.leds.into());
```

The `mk_static` macro is _slight_ black magic whose exact method of working is outside the scope of this tutorial, but it converts its second parameter into a static (mutable) reference of the type given as the first parameter. We need to do this because parameters to Embassy tasks must have a static lifetime.

Finally, we launch the task:

```rust
spawner.must_spawn(led_task(leds));
```

> [!TIP]
> An eagle-eyed coder might spot an oddity: what's going on here? Shouldn't that just evaluate `led_task(leds)` and then `spawner.must_spawn()` with its result value? Normally you would be correct, there's no Rust magic _here_ – instead, the [black magic](https://github.com/embassy-rs/embassy/blob/main/embassy-executor/src/lib.rs#L61) is done within the `#[task]` macro.

We add a delay loop at the end of `main` so we can see that two tasks are doing something:

```rust
loop {
    info!("Main task still alive");
    Timer::after_millis(5000).await;
}
```

## Running the code

When you run `cargo run --bin step_03_tasks`, you see that your LEDs blink and you should get messages in your terminal along the lines of:

`[INFO ] Main task still alive (step_03_tasks src/bin/step_03_tasks.rs:75)`

# Gently to the fourth step: let's add more everything!

Running one task with just a delay loop in another task is not really very interesting in practice, even though it's a major change in the underpinning architecture.

So let's make it more interesting [in the fourth step](004-buttons.md)!

# Suggested learning tasks

* Easy: Spawn another task that just logs messages using `info!`.
