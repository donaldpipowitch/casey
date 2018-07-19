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
    // Tracks cursor position (column).
    // This allows the user to navigate using the left/right arrow keys.
    cursor_pos: usize,
    // Tracks the line the user is currently typing on.
    // This is needed by the render() function.
    start_row: usize,
    // True if it should exit, false otherwise.
    done: bool,
}

impl State {
    fn new_(value: String, col: usize, row: usize, done: bool) -> State {
        State {
            value: value,
            cursor_pos: col,
            start_row: row,
            done: done,
        }
    }

    fn new(value: String, col: usize, row: usize) -> State {
        State::new_(value, col, row, false)
    }

    fn move_cursor(state: State, col_offset: isize, row_offset: isize) -> State {
        State::new_(state.value,
                    ((state.cursor_pos as isize) + col_offset) as usize,
                    ((state.start_row  as isize) + row_offset) as usize,
                    state.done)
    }

    fn done(state: State) -> State {
        State::new_(state.value, state.cursor_pos, state.start_row, true)
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
            state = State::move_cursor(state, 0, -1);
        }

        // Move the cursor to the start of the line, then print it.
        move_cursor(stdout, 0, state.start_row + i);
        write!(stdout, "{}", line).unwrap();
    }

    // Move the terminal's cursor to where we want it to be.
    move_cursor(stdout, state.cursor_pos, state.start_row);

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

fn update_state<W: Write>(mut stdout: &mut RawTerminal<W>, mut state: State,
                          key: Result<Key, Error>) -> State {
    match key.unwrap() {
        Key::Ctrl('c') => {
            // If there's text, jump to the lowercased line before
            // exiting, to avoid overwriting existing text.
            if !state.value.is_empty() {
                state = State::move_cursor(state, 0, 2);
            }
            return State::done(state);
        }
        Key::Char('\n') => {
            // If the user presses enter without any text, break
            // out of the for loop so we can exit.
            if state.value.is_empty() {
                return State::done(state);
            }

            move_cursor(&mut stdout, 0, state.start_row + 2);
            write!(stdout, "\n").unwrap();

            let (_total_cols, total_rows) = terminal_size().unwrap();
            let mut start_row = state.start_row + 2;
            if start_row as u16 != total_rows {
                start_row += 1;
            }

            return State::new(String::new(), 0, start_row);
        }
        Key::Char(key) => {
            let mut value = String::from(state.value);
            value.push(key);
            return State::new(value, state.cursor_pos + 1, state.start_row);
        }
        Key::Backspace => {
            if !state.value.is_empty() && state.cursor_pos != 0 {
                let str_value = &state.value[0..(state.cursor_pos - 1)];
                let value = String::from(str_value);

                return State::new(value, state.cursor_pos - 1,
                                  state.start_row);
            }
        }
        Key::Left => {
            if state.cursor_pos > 0 {
                return State::move_cursor(state, -1, 0)
            }
        }
        Key::Right => {
            if state.cursor_pos < state.value.len() - 1 {
                return State::move_cursor(state, 1, 0)
            }
        }
        _ => {}
    }

    return state;
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let (_col, row) = stdout.cursor_pos().unwrap();
    let mut state = State::new(String::new(), 0, row as usize);
    state = render(&mut stdout, state);

    let stdin = stdin();
    for key in stdin.keys() {
        state = update_state(&mut stdout, state, key);
        if state.done {
            break;
        }
        state = render(&mut stdout, state);
    }
}
