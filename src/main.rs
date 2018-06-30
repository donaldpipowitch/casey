extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::cursor::{DetectCursorPos, Goto};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, terminal_size};

#[derive(Debug)]
struct State {
    value: String,
    index: usize,
    start_row: usize,
    rows: usize,
}

impl State {
    fn new() -> State {
        State {
            value: String::new(),
            index: 0,
            start_row: 0,
            rows: 1,
        }
    }
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

fn render<W: Write>(stdout: &mut RawTerminal<W>, state: &mut State) {
    // clear all "known" rows
    for i in 0..state.rows {
        write!(
            stdout,
            "{}{}",
            Goto(1, state.start_row as u16 + i as u16),
            clear::CurrentLine
        ).unwrap();
    }

    // write
    let formatted = format_value(&state);
    for (i, line) in formatted.lines().enumerate() {
        let (_total_cols, total_rows) = terminal_size().unwrap();
        if state.start_row + i > total_rows as usize {
            write!(stdout, "\n").unwrap();
            state.start_row -= 1;
            state.rows += 1;
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
        Goto(state.index as u16 + 1, state.start_row as u16)
    ).unwrap();

    stdout.flush().unwrap();
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let (_col, start_row) = stdout.cursor_pos().unwrap();
    let mut state = State::new();
    state.start_row = start_row as usize;
    render(&mut stdout, &mut state);

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Ctrl('c') => {
                if !state.value.is_empty() {
                    // jump to lower cased line, before exiting
                    write!(stdout, "{}", Goto(1, state.start_row as u16 + 2)).unwrap();
                }
                break;
            }
            Key::Char('\n') => {
                if state.value.is_empty() {
                    break;
                } else {
                    let (_col, start_row) = stdout.cursor_pos().unwrap();
                    let (_total_cols, total_rows) = terminal_size().unwrap();
                    let end_of_screen = start_row + 2 == total_rows;

                    write!(stdout, "{}", Goto(1, state.start_row as u16 + 2)).unwrap();
                    write!(stdout, "\n").unwrap();
                    stdout.flush().unwrap();
                    state.value = String::new();

                    state.index = 0;
                    if end_of_screen {
                        state.start_row += 2;
                    } else {
                        state.start_row += 3;
                    }
                }
            }
            Key::Char(key) => {
                state.value.insert(state.index, key);
                state.index += 1;
            }
            // Key::Delete = entf
            Key::Backspace => {
                if !state.value.is_empty() && state.index != 0 {
                    state.value.remove(state.index - 1);
                    state.index -= 1;
                }
            }
            Key::Left => {
                if state.index > 0 {
                    state.index -= 1;
                }
            }
            Key::Right => {
                if state.index < state.value.len() - 1 {
                    state.index += 1;
                }
            }
            _ => {}
        }
        render(&mut stdout, &mut state);
    }
}
