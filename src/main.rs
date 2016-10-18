extern crate termion;

use std::io::{Write, stdout, stdin};
use std::env;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use std::path::Path;

mod coms;

#[derive(PartialEq)]
enum Operation {
    NoOp,
    Editing,
    GoingTo,
}

struct State {
    filename : String,
    field : (u32, u32),
    has_header : bool,
    data: Vec<Vec<String>>,
    cell_size: usize,
    editor_buffer : String,
    op : Operation,
}

use termion::{color, cursor, clear};


fn draw_mid_line(stdout : &mut std::io::Stdout, len : usize, state: &State) {
    let mut line : String = String::new();
    for i in (0..len+1) {
        if i == 0 {
            line.push('├');
        } else if i == len {
            line.push('┤');
        } else if i%state.cell_size == 0 {
            line.push('┼');
        } else {
            line.push('─');
        }
    }
    write!(stdout, "{}", line);
}

fn draw_bot_line(stdout : &mut std::io::Stdout, len : usize, state: &State) {
    let mut line : String = String::new();
    for i in (0..len+1) {
        if i == 0 {
            line.push('└');
        } else if i == len {
            line.push('┘');
        } else if i%state.cell_size == 0 {
            line.push('┴');
        } else {
            line.push('─');
        }
    }
    write!(stdout, "{}", line);
}

fn draw_top_line(stdout : &mut std::io::Stdout, len : usize, state: &State) {
    let mut line : String = String::new();
    for i in (0..len+1) {
        if i == 0 {
            line.push('┌');
        } else if i == len {
            line.push('┐');
        } else if i%state.cell_size == 0 {
            line.push('┬');
        } else {
            line.push('─');
        }
    }
    write!(stdout, "{}", line);
}

fn draw_header(stdout : &mut std::io::Stdout, state: &State) {
    let state = state.clone();
    write!(stdout, "{}{}",cursor::Goto(2,2), state.filename);
    let (width, height) = termion::terminal_size().unwrap();
    write!(stdout, "{}{},{}",cursor::Goto(width-20,2), state.field.0, state.field.1);
    write!(stdout, "{}{}",cursor::Goto(width-5,2), state.has_header);
}

fn draw_body(stdout : &mut std::io::Stdout, state: &State) {
    write!(stdout, "{}",cursor::Goto(2,4));
    let the_width = state.data[0].len() * state.cell_size;
    draw_top_line(stdout, the_width, &state);
    for (irow,row) in state.data.iter().enumerate() {
        for (icol, col) in row.iter().enumerate() {
            let colp = if col.len() < state.cell_size {
                col.to_string()
            } else {
                let mut new_str = String::new();
                new_str.push_str(&col[0..state.cell_size - 4].to_string());
                new_str.push_str(&"→");
                new_str
            };
            if irow == state.field.1 as usize && icol == state.field.0 as usize {
                write!(stdout, "{}│{}{}{}",cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),color::Fg(color::Yellow),colp,color::Fg(color::Reset)).unwrap();
            }
            else if irow == 0 && state.has_header == true {
                write!(stdout, "{}│{}{}{}",cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),color::Fg(color::Blue),colp,color::Fg(color::Reset)).unwrap();
            } else {
                write!(stdout, "{}│{}{}{}",cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),color::Fg(color::White),colp,color::Fg(color::Reset)).unwrap();
            }
            if icol == row.len() - 1 {
                write!(stdout, "{}│",cursor::Goto((the_width+2) as u16,5+(irow*2) as u16));
            }
        }
        // write!(stdout, "{}", cursor::Down(1)).unwrap();
        if irow != 0 {
            write!(stdout, "{}",cursor::Goto(2,4+(irow*2) as u16));
            draw_mid_line(stdout,the_width, &state);
        }
    }
    write!(stdout, "{}",cursor::Goto(2,4+(state.data.len() * 2) as u16));
    draw_bot_line(stdout,the_width, &state);
}

fn draw_editor(stdout : &mut std::io::Stdout, state: &State) {
    let (width, height) = termion::terminal_size().unwrap();
    write!(stdout, "{}Editor: {}",cursor::Goto(2,height-2), state.editor_buffer);
}

fn draw_window(stdout : &mut std::io::Stdout, state: &State) {
    write!(stdout,"{}{}",clear::All, cursor::Goto(1,1)).unwrap();
    draw_header(stdout, state);
    draw_body(stdout, state);
    if state.op == Operation::Editing {
        draw_editor(stdout, state);
    }
}

fn main() {
    use Operation;
    let mut state : State = State {
        filename : "".to_string(),
        has_header: false,
        field: (0,0),
        data: vec![],
        cell_size: 20,
        editor_buffer: "".to_string(),
        op: Operation::NoOp,
    };
    let args : Vec<String> = env::args().collect();
    let ref filename : String = args[1];
    state.filename = filename.to_string();
    let mut read_data = coms::read(Path::new(filename));
    state.data.append(&mut read_data);
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", cursor::Hide).unwrap();
    draw_window(&mut stdout, &state);
    stdout.flush().unwrap();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('q') => break,
            Key::Ctrl('h') => state.has_header = !state.has_header,
            Key::Ctrl('e') => {
                if state.op == Operation::NoOp {
                    state.op = Operation::Editing;
                    state.editor_buffer.clear();
                    let (x,y) = state.field;
                    state.editor_buffer.push_str(&state.data[y as usize][x as usize]);
                } else if state.op == Operation::Editing {
                    state.op = Operation::NoOp;
                    let (x,y) = state.field;
                    state.data[y as usize][x as usize] = state.editor_buffer.to_string();
                }
            },
            Key::Char(c) => {
                if state.op == Operation::Editing {
                    state.editor_buffer.push(c);
                }
            },
            Key::Backspace => {
                if state.op == Operation::Editing {
                    state.editor_buffer.pop();
                }
            },
            Key::Ctrl('s') => {
                // save
                ()
            }
            Key::Left => {
                if state.field.0 > 0 && state.op == Operation::NoOp {
                    state.field.0 -= 1
                }
            },
            Key::Right => {
                if state.op == Operation::NoOp {
                    state.field.0 += 1
                }
            },
            Key::Up => {
                if state.field.1 > 0 && state.op == Operation::NoOp {
                    state.field.1 -= 1;
                }
            },
            Key::Down => {
                if state.op == Operation::NoOp {
                    state.field.1 += 1;
                }
            },
            _ => ()
        }
        draw_window(&mut stdout, &state);
        stdout.flush().unwrap();
    }
    write!(stdout,"{}{}",clear::All, cursor::Show).unwrap();
}
