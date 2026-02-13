# Rust firmware tutorial for the Disobey 2026 badge

This is intended to be a tutorial for people who don't know much about Rust, ESP32, or embedded programming in general. Each step of the tutorial has its own buildable binary, starting with the "hello world" in `badge-firmware/src/bin/001-hello-world.rs`.

The biggest goal of this tutorial is to empower you to control your own hardware: you paid for your Disobey badge, and you can run anything you want on it!

There is also a secondary goal: to show that embedded programming has come very far even in the Rust world, and you don't need to (always) deal with esoteric low-level things: with a bit of automatically generated scaffolding, you have a lot of ready-made libraries and excellent tools available to you. This tutorial uses the [Embassy](https://embassy.dev/) framework which is so powerful that it makes embedded development with it feel like cheating!

> [!TIP]
> The tutorial is littered with small tips and asides like this. They are not required reading for this tutorial and are provided as interesting rabbit holes to fall into.

> [!TIP]
> If you are an experienced embedded Rust developer, you should be able to get started following the [cliff notes](https://github.com/tanelikaivola/disobey2026badge?tab=readme-ov-file#toolchain) at the BSP for the badge instead.

# Step 1: Required software

You will need to install three crucial pieces of software: the Rust toolchain itself with `rustup`, the ESP32-specific parts and versions of the Rust toolchain with `espup` (in this order), and `espflash`.

## rustup

Follow the installation instructions at [rustup's website](https://rustup.rs/). For *nix systems, there is a `curl | sh` command (yes, we know...), and for Windows there is an installer executable.

Rustup typically installs the Rust toolchain for your machine's architecture automatically. If you have previously installed the Rust toolchain with `rustup`, it's best to run `rustup update` just in case.

> [!CAUTION]
> `rustup` will install tools in `$HOME/.rustup`, `$HOME/.cargo`, and edit your shell configuration typically in `$HOME/.profile` for the current user. For most users this is not a concern, but if you don't like this and prefer to tweak all the knobs yourself, you can either configure `rustup` or install Rust via alternative means. The same applies for `espup` in the next step.

## Installing espup itself

Follow the installation instructions in [espup's README](https://github.com/esp-rs/espup), but if you have the pre-requisites installed it all boils down to `cargo install espup --locked`.

> [!NOTE]
> You might be wondering what this `cargo` thing is, you have never heard of it? It is Rust's package manager that `rustup` should have installed for you.

## Running espup to install the ESP Rust toolchain

In order to run `espup` **or compile any step of this tutorial** you need to run the following in your shell: `. $HOME/export-esp.sh`

This command needs to be run for every shell session, which means you might want to add it to your `.profile` or equivalent. See [espup's README](https://github.com/esp-rs/espup?tab=readme-ov-file#environment-variables-setup) for more details.

To install the Rust toolchain for ESP microcontrollers, run `espup install`.

## Installing espflash

To install `espflash`, simply run `cargo install espflash --locked`.

# Step 2: Compile the firmware

Connect your badge and in the `badge-firmware` directory of this repository, run the following: `cargo run`.

After watching the compilation process and the flashing process, you should finally see a line like the following one:

`[INFO ] Hello world! (step_01_hello_world src/bin/step_01_hello_world.rs:47)`

If you do, congratulations: you have compiled and flashed your own firmware in the badge – the message is being sent to your computer via the USB cable from the badge! You are now ready to **start following the tutorial in [`tutorial/001-hello-world.md`](tutorial/001-hello-world.md)**.

> [!NOTE]
> There should be nothing on the screen on the badge, and no LEDs should be on: you will need to wait until the second part of the tutorial for that.

If you get an error message along the lines of `error: linker xtensa-esp32s3-elf-gcc not found`, follow the instructions above regarding setting up `espup` in your shell. (That is, run `$HOME/export-esp.sh`.)

# Aside: How was this project generated?

You can build on the framework of this tutorial, but you might want to create your totally own binary later. In light of this: the Rust crate (project) in this repository was created using `esp-generate` with the following options:

* Board type: `esp32s3`
* Project name: `badge-firmware`
* Enable the following:
    * ✅ Enable unstable HAL features.
    * ✅ Enable allocations with the esp-alloc crate.
    * ✅ Enable Wi-Fi via the esp-radio crate.
    * ✅ Add embassy framework support.
    * Flashing, logging and debuggubg (espflash) -> ✅ Use defmt to print messages.
    * Optional editor integration -> ✅ Add settings for Visual Studio Code
    * Rust toolchain -> ✅ Use `esp` toolchain

# Licensing

The source code examples in the repository are licensed under the MIT license.

The Markdown files in the repository, excluding the quoted source code, are licensed under the CC-BY-NC-SA 4.0 license.

## MIT license

Copyright (c) 2026 Anssi Matti Helin

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## CC-BY-NC-SA 4.0

You are free to:

* Share — copy and redistribute the material in any medium or format
* Adapt — remix, transform, and build upon the material

The licensor cannot revoke these freedoms as long as you follow the license terms.

Under the following terms:

* Attribution — You must give appropriate credit, provide a link to the license, and indicate if changes were made. You may do so in any reasonable manner, but not in any way that suggests the licensor endorses you or your use.
* NonCommercial — You may not use the material for commercial purposes.
* ShareAlike — If you remix, transform, or build upon the material, you must distribute your contributions under the same license as the original.
* No additional restrictions — You may not apply legal terms or technological measures that legally restrict others from doing anything the license permits.

For the full legal terms, please see https://creativecommons.org/licenses/by-nc-sa/4.0/.