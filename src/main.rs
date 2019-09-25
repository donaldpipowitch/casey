extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::{clear,
              color,
              cursor::{DetectCursorPos, Goto},
              event::Key,
              input::TermRead,
              raw::{IntoRawMode, RawTerminal},
              terminal_size};

#[derive(Debug)]
struct State {
    // Contains the typed input from the user.
    value: String,
    // Tracks the cursor position (column).
    // This allows the user to navigate using the left/right arrow keys.
    cursor_pos: usize,
    // Tracks the line the user is currently typing on.
    // This is needed by the render() function.
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

// Renders the current state.
fn render<W: Write>(stdout: &mut RawTerminal<W>, state: &mut State) {
    // Clear rows so we have a clean "canvas" to work with.
    // The "canvas" is the current line and the following two lines.
    //
    // This basically "writes" clear::CurrentLine beginning
    // at the start_row and the next two rows.
    for i in 0..3 {
        write!(
            stdout,
            "{}{}",
            Goto(1, state.start_row as u16 + i as u16),
            clear::CurrentLine
        ).unwrap();
    }

    // Get the formatted output, and write it to the terminal.
    // NOTE: If the formatted output contains multiple lines, it will
    //       render every line separately, to avoid render bugs.
    let formatted = format_value(&state);
    for (i, line) in formatted.lines().enumerate() {
        // If we don't have enough space, due to being at the end
        // of the terminal screen, print newlines to create more space
        // and adjust `start_row` accordingly.
        let (_total_cols, total_rows) = terminal_size().unwrap();
        if state.start_row + i > total_rows as usize {
            write!(stdout, "\n").unwrap();
            state.start_row -= 1;
        }

        // Move the cursor to the start of the line, then print it.
        write!(
            stdout,
            "{}{}",
            Goto(1, state.start_row as u16 + i as u16),
            line
        ).unwrap();
    }

    // Move the terminal's cursor to where we want it to be.
    write!(
        stdout,
        "{}",
        Goto(state.cursor_pos as u16 + 1, state.start_row as u16)
    ).unwrap();

    // Flush the cache to ensure everything is printed.
    stdout.flush().unwrap();
}

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

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let (_col, start_row) = stdout.cursor_pos().unwrap();
    let mut state = State::new();
    state.start_row = start_row as usize;
    render(&mut stdout, &mut state);

    let stdin = stdin();
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Ctrl('c') => {
                if !state.value.is_empty() {
                    // jump to lower cased line, before exiting in an non-empty state
                    // so no line gets cropped
                    write!(stdout, "{}", Goto(1, state.start_row as u16 + 2)).unwrap();
                }
                break;
            }
            Key::Char('\n') => {
                if state.value.is_empty() {
                    break;
                } else {
                    write!(stdout, "{}\n", Goto(1, state.start_row as u16 + 2)).unwrap();

                    state.value = String::new();
                    state.cursor_pos = 0;
                    let (_total_cols, total_rows) = terminal_size().unwrap();
                    let end_of_screen = state.start_row as u16 + 2 == total_rows;
                    if end_of_screen {
                        state.start_row += 2;
                    } else {
                        state.start_row += 3;
                    }
                }
            }
            Key::Char(key) => {
                state.value.insert(state.cursor_pos, key);
                state.cursor_pos += 1;
            }
            Key::Backspace => {
                if !state.value.is_empty() && state.cursor_pos != 0 {
                    state.value.remove(state.cursor_pos - 1);
                    state.cursor_pos -= 1;
                }
            }
            Key::Left => {
                if state.cursor_pos > 0 {
                    state.cursor_pos -= 1;
                }
            }
            Key::Right => {
                if state.cursor_pos < state.value.len() - 1 {
                    state.cursor_pos += 1;
                }
            }
            _ => {}
        }
        render(&mut stdout, &mut state);
    }
}
