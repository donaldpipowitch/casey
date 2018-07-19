extern crate termion;

use std::io::{stdin, stdout, Error, Write};
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
    // Number of columns the terminal has.
    width: usize,
    // Number of rows the terminal has.
    height: usize,
    // Tracks cursor position (column).
    // This allows the user to navigate using the left/right arrow keys.
    col: usize,
    // Tracks the line the user is currently typing on.
    // This is needed by the render() function.
    row: usize,
    // True if it should exit, false otherwise.
    done: bool,
}

impl State {
    fn build(value: String, col: usize, row: usize, done: bool) -> State {
        let (width, height) = terminal_size().unwrap();

        State {
            value: value,
            width: width as usize,
            height: height as usize,
            col: col,
            row: row,
            done: done,
        }
    }

    fn new(value: String, col: usize, row: usize) -> State {
        State::build(value, col, row, false)
    }

    fn move_cursor(state: State, col_offset: isize, row_offset: isize) -> State {
        let mut col = (state.col as isize) + col_offset;
        let row = (state.row as isize) + row_offset;

        let len = state.value.len() as isize - 1;
        if col <= 0 {
            col = 1;
        } else if col > len {
            col = len;
        }

        State::build(state.value, col as usize, row as usize, state.done)
    }

    fn done(mut state: State) -> State {
        // If there's text, jump to the lowercased line before
        // exiting, to avoid overwriting existing text.
        if !state.value.is_empty() {
            state = State::move_cursor(state, 0, 2);
        }

        State::build(state.value, state.col, state.row, true)
    }

    fn key(state: State, key: char) -> State {
        let mut value = String::from(state.value);
        value.insert(state.col, key);
        State::new(value, state.col + 1, state.row)
    }

    fn backspace(state: State) -> State {
        if state.value.is_empty() || state.col == 0 {
            return state
        }

        let mut value = String::from(state.value);
        value.remove(state.col - 1);

        State::new(value, state.col - 1, state.row)
    }
}

// Move the cursor to the specified row and column.
fn move_cursor<W: Write>(stdout: &mut RawTerminal<W>, col: usize, row: usize) {
    let goto = Goto(col as u16 + 1, row as u16);
    write!(stdout, "{}", goto).unwrap();
}

// Move the cursor to the beginning of the specified row, and clear the line.
fn clear_row<W: Write>(stdout: &mut RawTerminal<W>, row: usize) {
    move_cursor(stdout, 0, row);
    write!(stdout, "{}", clear::CurrentLine).unwrap();
}

// Render the current state.
fn render<W: Write>(stdout: &mut RawTerminal<W>, mut state: State) -> State {
    // Clear rows so we have a clean "canvas" to work with.
    // The "canvas" is the current line and the following two lines.
    for i in 0..3 {
        clear_row(stdout, state.row + i);
    }

    // Get the formatted output.
    let formatted = format_value(&state);

    // Write it to the terminal.
    // NOTE: If the formatted output contains multiple lines, it will
    //       render every line separately to avoid render bugs.
    for (i, line) in formatted.lines().enumerate() {
        // If we don't have enough space, due to being at the end
        // of the terminal screen, print newlines to create more space
        // and adjust row accordingly.
        if state.row + i > state.height {
            write!(stdout, "\n").unwrap();
            state = State::move_cursor(state, 0, -1);
        }

        // Move the cursor to the start of the line, then print it.
        move_cursor(stdout, 0, state.row + i);
        write!(stdout, "{}", line).unwrap();
    }

    // Move the terminal's cursor to where we want it to be.
    move_cursor(stdout, state.col, state.row);

    // Flush the cache to ensure everything is printed.
    stdout.flush().unwrap();

    state
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

fn update_state(state: State, key: Result<Key, Error>) -> State {
    match key.unwrap() {
        Key::Ctrl('c')  => State::done(state),
        Key::Char('\n') => {
            if state.value.is_empty() {
                // If the user presses enter without any text, exit.
                State::done(state)
            } else {
                let mut row = state.row + 2;
                if row != state.height {
                    row += 1;
                }

                State::new(String::new(), 0, row)
            }
        }
        Key::Char(key)  => State::key(state, key),
        Key::Backspace  => State::backspace(state),
        Key::Left       => State::move_cursor(state, -1, 0),
        Key::Right      => State::move_cursor(state,  1, 0),
        _               => state,
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let (_col, row) = stdout.cursor_pos().unwrap();
    let mut state = State::new(String::new(), 0, row as usize);
    state = render(&mut stdout, state);

    let stdin = stdin();
    for key in stdin.keys() {
        state = update_state(state, key);
        if state.done {
            break;
        }
        state = render(&mut stdout, state);
    }
}
