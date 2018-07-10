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

```rs
extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, terminal_size};
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

---

Thanks for reading so far. I'd be happy to get feedback about this _"Tutorial as a `README.md`"_ format. It is an experiment to teach coding. I'd also be happy if you can point out.