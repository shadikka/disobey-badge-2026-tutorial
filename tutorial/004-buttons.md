# Step 4: Interactivity with buttons, multiple tasks, and asynchronous message passing

In this step we are starting to move closer to an architecture of an actually useful program: we have multiple tasks communicating with each other using an asynchronous pub/sub channel.

When the program from this step is flashed to the badge, the LEDs will change colour based on the button you press.

There is a lot of boilerplate in this step that could be shortened with relatively common helper macros, but we have written it all out for the sake of simplicity. So don't be too afraid!

## New imports

```
use embassy_futures::select::select_array;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
```

These imports are just building blocks for asynchronous communication between tasks.

> [!TIP]
> If you were adding these to the project yourself, you would use the following commands:
> 
> ```
> cargo add embassy-futures --features defmt
> cargo add embassy-sync --features defmt
> ```

## Button press enum

We create an enumeration type for which button was pressed to be sent between tasks:

```rust
#[derive(Clone, Copy, defmt::Format)]
enum ButtonPressEvent {
    Up,
    Down,
    Left,
    Right,
    Stick,
    A,
    B,
    Start,
    Select,
}
```

There is nothing particularly interesting here except the `derive` macro: we make the type cloneable (which can be an O(anything) operation), copyable (which should be an O(1) operation), and formattable as text in anything using `defmt` – like our serial terminal!

## Channel definition and type shorthands

We then create our static pub/sub channel (multiple publishers, multiple subscribers), and to make the type boilerplate a bit more manageable we also create some type aliases.

```rust
static BUTTON_CHANNEL: PubSubChannel<CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1> = PubSubChannel::new();
type ButtonSubscriber = Subscriber<'static, CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1>;
type ButtonPublisher = Publisher<'static, CriticalSectionRawMutex, ButtonPressEvent, 8, 2, 1>;
```

These might look scary, but let's again take a deep breath and go through it piece by piece: our `BUTTON_CHANNEL` is:
* A `PubSubChannel` which has [five type parameters](https://docs.embassy.dev/embassy-sync/git/default/pubsub/struct.PubSubChannel.html) which are the next five parts:
    * Backed by a `CriticalSectionRawMutex`
    * Transmitting `ButtonPressEvent`s
    * With space for (an arbitrarily chosen) 8 elements
    * With two subscribers
    * With one publisher

Similarly, our `ButtonSubscriber` and `ButtonPublisher` type aliases are the same, but they also have the `'static` lifetime specifier: they can be only used for channels which are alive for the entirety of our program. Luckily we just created our channel as `static`!

## Palette extension

Because we want one colour per button, we extend our palette a bit and change its name from `RAINBOW` to a more generic `PALETTE`:

```rust
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
```

## LED task changes

Our LED task has changed slightly: first off in our function signature we get another parameter which is a subscriber to our channel:

```rust
async fn led_task(mut subscriber: ButtonSubscriber, leds: &'static mut Leds<'static>) {
```

Then, instead of getting the next colour from the rainbow iterator, we get it from the event sent via the channel, and instead of waiting for a specific duration has passed we wait until there is a new message.

```rust
let event = subscriber.next_message_pure().await;
// This is purposefully verbose for the sake of simplicity here.
// Normally we would use something like the `num_enum` crate instead.
let color = match event {
    ButtonPressEvent::Up => PALETTE[0],
    ButtonPressEvent::Down => PALETTE[1],
    ButtonPressEvent::Left => PALETTE[2],
    ButtonPressEvent::Right => PALETTE[3],
    ButtonPressEvent::Stick => PALETTE[4],
    ButtonPressEvent::A => PALETTE[5],
    ButtonPressEvent::B => PALETTE[6],
    ButtonPressEvent::Start => PALETTE[7],
    ButtonPressEvent::Select => PALETTE[8],
};
```

Note the comment about the verbosity – there are plenty of helper crates for doing something like this in Rust safely!

## New button task

Our button task is pretty much just one large `match` statement inside a loop. We have a common `select` paradigm: we wait for one of an array of `Future` types to be ready, or in other words, for any of the buttons to be pressed.

```rust
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
    ((), 0) => publisher.publish(ButtonPressEvent::Up).await,
    ((), 1) => publisher.publish(ButtonPressEvent::Down).await,
    ((), 2) => publisher.publish(ButtonPressEvent::Left).await,
    ((), 3) => publisher.publish(ButtonPressEvent::Right).await,
    ((), 4) => publisher.publish(ButtonPressEvent::Stick).await,
    ((), 5) => publisher.publish(ButtonPressEvent::A).await,
    ((), 6) => publisher.publish(ButtonPressEvent::B).await,
    ((), 7) => publisher.publish(ButtonPressEvent::Start).await,
    ((), 8) => publisher.publish(ButtonPressEvent::Select).await,
    _ => unreachable!(),
}
```

As you can see, we are using the convenience function `Buttons::debounce_press` which will only trigger if a button is pressed for 20 ms – as physical switches are unfortunately physical and have real-world limitations, it is not at all uncommon for a microswitch to trigger multiple times for just one button press.

## Main function changes

In our `main` function, we have added a channel subscriber for the `main` function itself, and the new parameters for the tasks:

```rust
let mut subscriber = BUTTON_CHANNEL.subscriber().unwrap();
spawner.must_spawn(led_task(BUTTON_CHANNEL.subscriber().unwrap(), leds));
spawner.must_spawn(button_task(BUTTON_CHANNEL.publisher().unwrap(), buttons));
```

> [!TIP]
> I'm really reaching you bad habits here, but this is another case where we know that we can safely `.unwrap()` the calls to `.subscriber()` and `.publisher()` because they only return errors if we exceed the subscriber or publisher counts in the type definition.

Finally, we just have a simple loop waiting for button press events and outputting them to the terminal:

```rust
let message = subscriber.next_message_pure().await;
info!("Main received message: {:?}", message);
```

## Running the code

Run the code with `cargo run --bin step_04_buttons`, press some buttons on the badge and enjoy the pretty colours.

# Fifth step: draw the rest of the owl

So far we have not touched our biggest and prettiest peripheral: our OLED display! So, in the fifth and final step of this tutorial, we finally draw (the rest of the) owl on it.

**[Go to the fifth step](005-display.md)!**