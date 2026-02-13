# Step 5: Draw an owl

Here we are, at the end of this tutorial. After this step, you are a fully fledged embedded Rust guru, able to conquer the world and bend all hardware to your whims.

Or at least, I hope, you have some level of basic understanding how to program the Disobey 2026 badge using Rust and the Embassy framework.

The program adds one new feature to the previous steps: an owl(\*) on the display which moves when you press the joystick left or right.

(\* The author shall receive no artistic critique of the rendition of the owl unless said critique is attached to a pull request improving the owl – in other words, your final assignment is to [draw the rest of the owl](https://knowyourmeme.com/memes/how-to-draw-an-owl).)

## New imports

```rust
use embedded_graphics::{
    mono_font::{MonoTextStyle, iso_8859_1::FONT_10X20},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle, StyledDrawable},
    text::Text,
};
```

Lots of imports from the `embedded_graphics` crate! Let's look at some of them in particular

* `mono_font::iso_8859_1::FONT_10X20` is a monospaced font, covering the ISO-8859-1 character set, with a 10x20 pixel size
* `pixelcolor::Rgb565` is an encoding for colours with a 5-bit red, 6-bit green, and a 5-bit blue channel, used for our display

The rest of the imports should be relatively self-describing.

> [!TIP]
> You can probably almost guess how you would install this yourself: `cargo add embedded-graphics --features defmt`

## Type changes for the channel

We want one additional subscriber to the pub/sub channel so we simply change the `2` to a `3` in the types:

```rust
static BUTTON_CHANNEL: PubSubChannel<CriticalSectionRawMutex, ButtonPressEvent, 8, 3, 1> = PubSubChannel::new();
```

And similarly for the `ButtonSubscriber` and `ButtonPublisher` type aliases.

## The display task

The display task will not be quoted here as-is due to its silly length, but let's go through some of the key parts:

```rust
// Blank the display
display
    .fill_solid(
        &Rectangle::new(Point::new(0, 0), Size::new(320, 170)),
        Rgb565::BLACK,
    )
    .unwrap_or_else(|_| warn!("Unable to blank display"));
```

The display module has a framebuffer of its own – and if we don't blank it, you will get an interesting visual experience typically only seen when consuming substances illegal in most Western jurisdictions.

> [!TIP]
> This function contains a lot of frankly quite ugly Rust code. Normally you would wrap the primitives to be drawn in a list or an array and draw them in one go, you would **not** wildly cast `u32` as `i32` and so on: while this code works, it is not what you might call idiomatic Rust just for the sake of simplicity and being easy to edit.

Drawing text is slightly convoluted, but here we specify the text, calculate its X coordinate so its horizontally centred on the display, specify the font and colour, and finally draw it:

```rust
let text = "HELLO I AM AN OWL";
let text_x: i32 = 160 - (text.len() as i32 * 10) / 2;
let text_pos = Point::new(text_x, TEXT_Y);
let text_style = MonoTextStyle::new(&FONT_10X20, TEXT_COLOR);
Text::new(text, text_pos, text_style)
    .draw(display)
    .unwrap_or_else(|_| {
        warn!("Unable to draw text :(");
        text_pos
    });
```

This time we are even properly using an `.unwrap_or_else()` call which gracefully handles an error by giving us a warning on our serial terminal.

```rust
let event = subscriber.next_message_pure().await;
// one line removed as not relevant
match event {
    ButtonPressEvent::Left => {
        owl_x = (owl_x.saturating_sub(1)).max(OWL_MIN_X);
    }
    ButtonPressEvent::Right => {
        owl_x = (owl_x.saturating_add(1)).min(OWL_MAX_X);
    }
    _ => {}
}
```

We use a `match` pattern to care only about `ButtonPressEvent`s signifying a joystick press either to the left or the right. We then set `owl_x` to the new value, nicely handling (literal) edge cases by just having a maximum and minimum coordinate for the owl.

```rust
// Clear old owl position
let clear_area = Rectangle::new(
    Point::new(owl_x, OWL_Y),
    Size::new(OWL_BODY_DIAMETER, OWL_BODY_DIAMETER + OWL_HEAD_DIAMETER),
);
display
    .fill_solid(&clear_area, Rgb565::BLACK)
    .unwrap_or_else(|_| warn!("Unable to clear old owl"));
```

As the comment suggests, we then clear the area that contained the old owl by drawing a black rectangle on it, before proceeding to painstakingly draw the most beautiful pixel owl ever seen on the Disobey 2026 badge (as of typing this guide).

```rust
// Draw new owl
let owl_head_middle_x = owl_x
    + (OWL_BODY_DIAMETER as i32 - OWL_HEAD_DIAMETER as i32) / 2
    + OWL_HEAD_DIAMETER as i32 / 2;
Circle::new(
    Point::new(owl_x, OWL_Y + OWL_HEAD_DIAMETER as i32),
    OWL_BODY_DIAMETER,
)
.draw_styled(&OWL_STYLE, display)
.unwrap_or_else(|_| warn!("Unable to draw owl body"));
// 6 similar circle and line call chains omitted for brevity
```

## Minor changes to main

In our `main` function, all we need to change is to create the static reference to our display using `mk_static!` and spawning the display task:

```rust
let display = mk_static!(Display, resources.display.into());
// ...
spawner.must_spawn(display_task(BUTTON_CHANNEL.subscriber().unwrap(), display));
```

Et voilà!

## Suggested learning tasks

* Easy: Change the text.
* Easy: Change the background and foreground colours of the owl.
* Easy(?): Draw the rest of the owl.
* Medium: Make the owl move on the Y axis as well.
* Hard: Change the background colour automatically on a timer, mimicking the `nametag` example in the `disobey2026badge` repository.
