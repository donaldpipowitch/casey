# `casey`

> A simple command line tool to uppercase and lowercase strings.

This `README.md` is written as a tutorial. If you're just interested in using this package, you'll find the usage information first. In the second part I'll explain how this package was created. This should be helpful, if you want to create a similar package or if you want to contribute to this package.

## Usage

To use this program make sure you have [Rust installed with `rustup`](https://www.rust-lang.org) (and you'll also need [Git](https://git-scm.com/)). This program uses stable Rust. Run the following steps to download the source code, compile it and run the program:

```bash
$ git clone git@github.com:donaldpipowitch/casey.git
$ cd casey
$ cargo run
```

You can than start to type any string. This string will be camel cased immediately in the following line and lower cased in the line after that. Note that this program currently doesn't handle multilines well.

Here is an example of what you would see:

![an example of a running Casey session](./hello-world.gif)

## Contribute

In this section I want to take about how this program was created, so you are able to create similar projets, fork this project or contribute back to it. I'm happy if you can point out any spelling mistakes as I'm not a native english speaker. In general you're allowed to contribute features back to this project (e.g. like multiline support), but keep in mind that I'll only accepts pull requests if this _"Contribute"_ section is kept in sync with the changes. I hope you'll enjoy the read 👋

Before we start on word of warning. I found some parts of this code to be quite "flaky". Take [this commit](https://github.com/donaldpipowitch/casey/commit/6df76a451275e1834b280c22702d420164cd738e) as an example. I'd expect that the deleted line behaves like the two new lines - and on Windows they actually did. But not on my Mac. There are probably a lot more edge cases like that one, so take everything with a grain of salt.

To start make sure to have the same prequisites as mentioned in the _"Usage"_ section, namigly have [Rust installed with `rustup`](https://www.rust-lang.org).

I started the project by creating an empty directory and adding some basic files to it for my initial setup.

The [`.gitignore`](.gitignore) is the same we get by running `$ cargo init` (see [here](https://git-scm.com/docs/gitignore), if you want to know more about `.gitignore` files):

```
/target
**/*.rs.bk
```

The [`Cargo.toml`](.Cargo.toml) is _nearly_ the same we get by running `$ cargo init` (see [here](https://doc.rust-lang.org/cargo/reference/manifest.html), if you want to know more about `Cargo.toml` files). The one thing which was added was [`termion`](https://gitlab.redox-os.org/redox-os/termion) as our only dependency. `termion` is needed to create a REPL-like application. As mentioned in the _"Usage"_ section we want to _immediately_ show some transformed output of the user input. `termion` will cover the needs for this like clearing lines or moving the cursor in the terminal. [Because of a bug](https://gitlab.redox-os.org/redox-os/termion/issues/140) I actually specified to use the Git source of this crate. (Thank you @JoshMcguigan for the bug fix!):

```
[package]
name = "casey"
version = "0.1.0"
authors = ["Donald Pipowitch <pipo@senaeh.de>"]

[dependencies]
termion = {git = "https://github.com/redox-os/termion"}
```

The whole application was created in the `src/main.rs` file. First we'll import a couple of modules which we need for handling the terminal. Most of them should be quite self-explanatory. If you don't know them - no worry. You'll see how they are used in a bit.

```rust
extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::{clear,
              color,
              cursor::{DetectCursorPos, Goto},
              event::Key,
              input::TermRead,
              raw::{IntoRawMode, RawTerminal},
              terminal_size};
```

Before we move on I'll give you an overview about the logic that we will implement. Let us recap our requirements:
- The user should be able to type something to the terminal.
- We'll immediately uppercase and lowercase the input and show the result _below_ the input of the user.
- Because we ignore line breaks for now, this will give us a total of three lines which we'll show: the input, the uppercased value and the lowercased value.
- And to make everything a little bit more interesting:
  - There will be an empty state which tells the user to start typing.
  - You can remove a character by pressing delete and you can use the left and right arrows for navigation.
  - By pressing Enter we'll "save" the current value and start typing on a completely new line or exit if the value is empty.
  - You can exit via Ctrl+C.

To achieve that we'll use a "state-render-pattern" which says: We have some state which can change based on some events and every time the state changes, we'll render our application. You might already be familiar with something like that from frontend development where this pattern (in different flavors) is currently quite popular.

So we need some state which will be a struct and we need some render function and both of them are glued together in our `main` function. So the high level overview would be:

```rust
struct State {
    // contains all the data we need for rendering the application
}

fn render(state: &State) { // not the real signature! it's simplified here
    // the render function renders our application based on the given state
}

fn main() {
    // create initial state
    // react on events to change state
    // call render and pass the state
}
```

Let's begin with the state. I just copy and paste the relevant code here where its fields are explained in the comments:

```rust
#[derive(Debug)]
struct State {
    // contains the typed input of the user
    value: String,
    // tracks the cursor position (so the user can navigate with the
    // left and right arrow keys within the input)
    cursor_pos: usize,
    // helps to keep track of the line where the user currently types
    // this is partially needed by the render function as well
    start_row: usize,
}

impl State {
    fn new() -> State {
        State {
            value: String::new(),
            cursor_pos: 0,
            start_row: 0,
        }
    }
}
```

The one thing to note should the `start_row` field. This one is purely needed for the rendering function as it sometimes need to know where the current input happens, so other lines can be cleared and we can jump back to this line. That's why the `render` function currently needs a mutable reference of the state - as you'll see in a minute. I'd consider this as a technical debt, but it is okay for now. A good refactoring action would be to split our current `State` into an `ApplicationState` which is needed for the application logic and as read-only data for the `render` function and into a `RenderState` which only the `render` function really needs to know about.

Let's have a look at the `render` function. This one is actually the most complex part, because rendering to the terminal can be quite tricky and buggy. I couldn't find a nice API for all my use cases. One example: To clear a line you don't call a `clear_line` function and pass the index of the line to the function, but you have to place the cursor on the line you want to clear, than clear the current line and after that restore the cursor to its previous position - and this all happens by _writing_ to the terminal. _Ouch!_ If you know a nicer API, please make a pull request. The basic logic of the render function looks like this:
- clear all lines to get a solid base (the user input, the uppercased output and the lowercased output - if the last two aren't yet available, it's not a problem, because we would just clear an empty line)
- write the new lines based on the (changed) state
- restore cursor position 💁‍
- flush everything to the terminal (the _actual_ rendering in the terminal happens here)

Besides the state we need to pass a `RawTerminal` to the `render` function. This is basically our canvas which will be written by the `render`.

```rust
fn render<W: Write>(stdout: &mut RawTerminal<W>, state: &mut State) {
    // clear rows to have a solid base
    // this basically "writes" clear::CurrentLine beginning
    // at the start_row and the next two rows
    for i in 0..3 {
        write!(
            stdout,
            "{}{}",
            Goto(1, state.start_row as u16 + i as u16),
            clear::CurrentLine
        ).unwrap();
    }

    // get the formatted output and write it to the terminal
    // if the formatted output contains multiple lines, it will
    // render every line separately (to avoid some render bugs)
    let formatted = format_value(&state);
    for (i, line) in formatted.lines().enumerate() {
        // if we don't have enough space, because we're at the end
        // of the terminal screen we need to create a new line
        // and adjust the start_row
        let (_total_cols, total_rows) = terminal_size().unwrap();
        if state.start_row + i > total_rows as usize {
            write!(stdout, "\n").unwrap();
            state.start_row -= 1;
        }
        write!(
            stdout,
            "{}{}",
            Goto(1, state.start_row as u16 + i as u16),
            line
        ).unwrap();
    }

    // update cursor
    write!(
        stdout,
        "{}",
        Goto(state.cursor_pos as u16 + 1, state.start_row as u16)
    ).unwrap();

    stdout.flush().unwrap();
}
```

One thing I want to highlight again: This all seems to be quite flaky currently. As far as I know it should be fine to write a string to `stdout` which contains multiple new lines, but it doesn't work correctly for me on the machines where I tested my app. Sometimes a line was missing or wasn't cleared or the cursor position got messed upt. To get rid of all bugs, I actually needed to write every line separately.

As you can see the `render` function calls a function I haven't previously talked about: `format_value`. It was previously part of the `render` function, but I extracted it out. One could argue that this would be the _real_ rendering function. It takes the _immutable_ state as its input and returns the string as the output. It doesn't know about the terminal and could generate a website in the same way for example. This function shows either an empty state or the state with the uppercased and lowercased output. Nicely formatted with some slight coloring.

```rust
fn format_value(state: &State) -> String {
    if state.value.is_empty() {
        format!(
            "{grey}Start typing...{reset}",
            grey = color::Fg(color::LightBlack),
            reset = color::Fg(color::Reset)
        )
    } else {
        format!(
            "{value}\n{grey}{uppercase}\n{lowercase}{reset}",
            value = state.value,
            grey = color::Fg(color::LightBlack),
            uppercase = state.value.to_uppercase(),
            lowercase = state.value.to_lowercase(),
            reset = color::Fg(color::Reset),
        )
    }
}
```

---

Thanks for reading so far. I'd be happy to get feedback about this _"Tutorial as a `README.md`"_ format. It is an experiment to teach coding. I'd also be happy if you can point out.