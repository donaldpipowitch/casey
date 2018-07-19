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

// Move the cursor to the specified row and column.
fn move_cursor<W: Write>(stdout: &mut RawTerminal<W>, col: usize, row: usize) {
    let goto = Goto(col as u16 + 1, row as u16);
    write!(stdout, "{}", goto).unwrap();
}

// Move the cursor to column zero of the
// specified row, and clear the line.
fn clear_row<W: Write>(stdout: &mut RawTerminal<W>, row: usize) {
    move_cursor(stdout, 0, row);
    write!(stdout, "{}", clear::CurrentLine).unwrap();
}

// Render the current state.
fn render<W: Write>(stdout: &mut RawTerminal<W>, state: &mut State) {
    // Clear rows so we have a clean "canvas" to work with.
    // The "canvas" is the current line and the following two lines.
    for i in 0..3 {
        clear_row(stdout, state.start_row + i);
    }

    // Get the formatted output.
    let formatted = format_value(&state);

    // Write it to the terminal.
    // NOTE: If the formatted output contains multiple lines, it will
    //       render every line separately to avoid render bugs.
    for (i, line) in formatted.lines().enumerate() {
        let (_total_cols, total_rows) = terminal_size().unwrap();

        // If we don't have enough space, due to being at the end
        // of the terminal screen, print newlines to create more space
        // and adjust start_row accordingly.
        if state.start_row + i > total_rows as usize {
            write!(stdout, "\n").unwrap();
            state.start_row -= 1;
        }

        // Move the cursor to the start of the line, then print it.
        move_cursor(stdout, 0, state.start_row + i);
        write!(stdout, "{}", line).unwrap();
    }

    // Move the terminal's cursor to where we want it to be.
    move_cursor(stdout, state.cursor_pos, state.start_row);

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
                // If there's text, jump to the lowercased line before
                // exiting, to avoid overwriting existing text.
                if !state.value.is_empty() {
                    move_cursor(&mut stdout, 0, state.start_row + 2);
                }
                break;
            }
            Key::Char('\n') => {
                // If the user presses enter without any text, break
                // out of the for loop so we can exit.
                if state.value.is_empty() {
                    break;
                }

                move_cursor(&mut stdout, 0, state.start_row + 2);
                write!(stdout, "\n").unwrap();

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
