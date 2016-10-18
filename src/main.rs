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
    Resizing,
}

#[derive(PartialEq)]
enum SaveState {
    Edited,
    Saved,
}

struct State {
    filename : String,
    field : (u32, u32),
    has_header : bool,
    data: Vec<Vec<String>>,
    cell_size: usize,
    editor_buffer : String,
    op : Operation,
    save_state : SaveState,
    goto_buffer : String,
    resize_buffer : String,
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

fn draw_goto(stdout : &mut std::io::Stdout, state: &State) {
    let (width, height) = termion::terminal_size().unwrap();
    write!(stdout, "{}Going to: {}",cursor::Goto(2,height-2), state.goto_buffer);
}

fn draw_resize(stdout : &mut std::io::Stdout, state: &State) {
    let (width, height) = termion::terminal_size().unwrap();
    write!(stdout, "{}New size: {}",cursor::Goto(2,height-2), state.resize_buffer);
}

fn draw_save_state(stdout : &mut std::io::Stdout, state: &State) {
    let (width, height) = termion::terminal_size().unwrap();
    match state.save_state {
        SaveState::Edited => {
            write!(stdout, "{}edited",cursor::Goto(width-6,height-2));
        },
        SaveState::Saved => {
            write!(stdout, "{}saved",cursor::Goto(width-6,height-2));
        },
    }
}

fn draw_window(stdout : &mut std::io::Stdout, state: &State) {
    write!(stdout,"{}{}",clear::All, cursor::Goto(1,1)).unwrap();
    draw_header(stdout, state);
    draw_body(stdout, state);
    if state.op == Operation::Editing {
        draw_editor(stdout, state);
    } else if state.op == Operation::GoingTo {
        draw_goto(stdout, state);
    } else if state.op == Operation::Resizing {
        draw_resize(stdout, state);
    }
    draw_save_state(stdout, state);
}

fn resize_data(data: &mut Vec<Vec<String>>, width: usize, height: usize) {
    let row_number = data.len();
    let col_number = data[0].len();
    // let mut data_copy : Vec<Vec<String>> = data.iter().map(|d| d.resize(width, "/".to_string()) ).collect::<Vec<Vec<String>>>();
    for row in &mut data.iter_mut() {
        row.resize(width, "/".to_string());
    }
    data.resize(height, vec!["/".to_string() ; width]);
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
        save_state: SaveState::Saved,
        goto_buffer : "".to_string(),
        resize_buffer : "".to_string(),
    };
    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        return ()
    }
    let ref filename : String = args[1];
    state.filename = filename.to_string();

    if coms::check(Path::new(filename)) {
        // file exists
        let mut read_data = coms::read(Path::new(filename));
        state.data.append(&mut read_data);
    } else {
        // file does not exist
        let mut temporary_data : Vec<Vec<String>> = vec![];
        temporary_data.push(vec!["Empty".to_string()]);
        temporary_data.push(vec!["0".to_string()]);
        state.data.append(&mut temporary_data);
    }

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
                    state.save_state = SaveState::Edited;
                }
            },
            Key::Ctrl('u') => {
                if state.op == Operation::Editing {
                    state.editor_buffer.clear();
                }
            }
            Key::Ctrl('g') => { // goto
                if state.op == Operation::NoOp {
                    state.op = Operation::GoingTo;
                    state.goto_buffer.clear();
                } else if state.op == Operation::GoingTo {
                    state.op = Operation::NoOp;
                    let params : Vec<&str> = state.goto_buffer.split(",").collect();
                    let nums : Vec<u32> = params.iter().map(|p| {p.parse::<u32>().unwrap()}).collect();
                    if nums.len() == 2 {
                        state.field = (nums[0], nums[1]);
                    } else {
                        println!("GOTO: Not enough parameters.");
                    }
                }
            },
            Key::Ctrl('r') => {
                // resize
                if state.op == Operation::NoOp {
                    state.op = Operation::Resizing;
                    state.resize_buffer.clear();
                } else if state.op == Operation::Resizing {
                    state.op = Operation::NoOp;
                    // resize here
                    let params : Vec<&str> = state.resize_buffer.split(",").collect();
                    let nums : Vec<usize> = params.iter().map(|p| {p.parse::<usize>().unwrap()}).collect();
                    if nums.len() == 2 {
                        resize_data(&mut state.data, nums[0], nums[1])
                    } else {
                        println!("RESIZE: Wrong number of parameters.");
                    }
                }
            }
            Key::Char(c) => {
                if state.op == Operation::Editing {
                    state.editor_buffer.push(c);
                } else if state.op == Operation::GoingTo {
                    state.goto_buffer.push(c);
                } else if state.op == Operation::Resizing {
                    state.resize_buffer.push(c);
                }
            },
            Key::Backspace => {
                if state.op == Operation::Editing {
                    state.editor_buffer.pop();
                } else if state.op == Operation::GoingTo {
                    state.goto_buffer.pop();
                } else if state.op == Operation::Resizing {
                    state.resize_buffer.pop();
                }
            },
            Key::Ctrl('s') => { // save
                coms::write(&state.data, Path::new(filename));
                state.save_state = SaveState::Saved;
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
