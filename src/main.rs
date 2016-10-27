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
    for i in 0..len+1 {
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
    write!(stdout, "{}", line).unwrap();
}

fn draw_bot_line(stdout : &mut std::io::Stdout, len : usize, state: &State) {
    let mut line : String = String::new();
    for i in 0..len+1 {
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
    write!(stdout, "{}", line).unwrap();
}

fn draw_top_line(stdout : &mut std::io::Stdout, len : usize, state: &State) {
    let mut line : String = String::new();
    for i in 0..len+1 {
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
    write!(stdout, "{}", line).unwrap();
}

fn draw_header(stdout : &mut std::io::Stdout, state: &State) {
    let state = state.clone();
    write!(stdout, "{}{}",cursor::Goto(2,2), state.filename).unwrap();
    let (width, _) = termion::terminal_size().unwrap();
    write!(stdout, "{}{},{}",cursor::Goto(width-20,2), state.field.0, state.field.1).unwrap();
    write!(stdout, "{}{}",cursor::Goto(width-5,2), state.has_header).unwrap();
}

fn draw_body(stdout : &mut std::io::Stdout, state: &State) {
    write!(stdout, "{}",cursor::Goto(2,4)).unwrap();
    let (width, height) = termion::terminal_size().unwrap();
    let the_width = state.data[0].len() * state.cell_size;
    let the_height = (state.data.len() * 2) + 1;

    let resized_height = if the_height < ((height - 6) as usize) {
        the_height as u16
    } else {
        (height - 6) as u16
    };

    let resized_width = if the_width < (width as usize) {
        the_width as u16
    } else {
        width - (width % (state.cell_size as u16))
    };

    let resized_cell_number = resized_width / (state.cell_size as u16);
    let resized_row_number = resized_height / 2;

    // skip, take

    let nortd = resized_row_number as usize; // number of rows to draw
    let noctd = resized_cell_number as usize; // number of cols to draw

    let cols_to_skip = if (((state.field.0 as i16) + 1) - (noctd as i16)) > 0 {
        ((state.field.0 + 1) - noctd as u32)
    } else {
        0
    };

    let rows_to_skip = if (((state.field.1 as i16) + 1) - (nortd as i16)) > 0 {
        ((state.field.1 + 1) - nortd as u32)
    } else {
        0
    };

    draw_top_line(stdout, resized_width as usize, &state);
    for (irow,row) in state.data.iter().skip(rows_to_skip as usize).take(nortd).enumerate() {
        for (icol, col) in row.iter().skip(cols_to_skip as usize).take(noctd).enumerate() {
            let colp = if col.len() < state.cell_size { // shortened
                col.to_string()
            } else {
                let mut new_str = String::new();
                new_str.push_str(&col[0..state.cell_size - 4].to_string());
                new_str.push_str(&"→");
                new_str
            };

            if (irow + rows_to_skip as usize) == state.field.1 as usize && (icol + cols_to_skip as usize) == state.field.0 as usize { // selected
                write!(stdout, "{}│{}{}{}",
	                cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),
	                color::Fg(color::Yellow),
	                colp,color::Fg(color::Reset)
                ).unwrap();
            }
            else if irow == 0 && state.has_header == true && rows_to_skip == 0 { // header
                write!(stdout, "{}│{}{}{}",
	                cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),
	                color::Fg(color::Blue),
	                colp,color::Fg(color::Reset)
               	).unwrap();
            } else { // normal
                write!(stdout, "{}│{}{}{}",
	                cursor::Goto(2+(icol * state.cell_size) as u16, 5+(irow*2) as u16),
	                color::Fg(color::White),
	                colp,color::Fg(color::Reset)
                ).unwrap();
            }

            if icol == noctd - 1 {
                write!(stdout, "{}│",cursor::Goto((resized_width+2) as u16,5+(irow*2) as u16)).unwrap();
            }
        }
        // write!(stdout, "{}", cursor::Down(1)).unwrap();
        if irow != 0 {
            write!(stdout, "{}",cursor::Goto(2,4+(irow*2) as u16)).unwrap();
            draw_mid_line(stdout,resized_width as usize, &state);
        }
    }
    draw_arrows(stdout,
        (((state.field.1 as i16) + 1) - (nortd as i16)),
        (((state.field.0 as i16) + 1) - (noctd as i16)),
        2,
        height - 2
    );
    // write!(stdout, "{}",cursor::Goto(2,4+(state.data.len() * 2) as u16));
    write!(stdout, "{}",cursor::Goto(2, ((nortd * 2)+4) as u16)).unwrap();
    draw_bot_line(stdout,resized_width as usize, &state);
}

fn draw_entry(stdout : &mut std::io::Stdout, state: &State) {
    let (_, height) = termion::terminal_size().unwrap();
    match state.op {
        Operation::Editing => {
            write!(stdout, "{}Editor: {}",cursor::Goto(2,height-2), state.editor_buffer).unwrap();
        },
        Operation::GoingTo => {
            write!(stdout, "{}Going to: {}",cursor::Goto(2,height-2), state.goto_buffer).unwrap();
        },
        Operation::Resizing => {
            write!(stdout, "{}New size: {}",cursor::Goto(2,height-2), state.resize_buffer).unwrap();
        },
        _ => ()
    }
}

fn draw_save_state(stdout : &mut std::io::Stdout, state: &State) {
    let (width, height) = termion::terminal_size().unwrap();
    match state.save_state {
        SaveState::Edited => {
            write!(stdout, "{}edited",cursor::Goto(width-6,height-2)).unwrap();
        },
        SaveState::Saved => {
            write!(stdout, "{}saved",cursor::Goto(width-6,height-2)).unwrap();
        },
    }
}

fn draw_arrows(stdout: &mut std::io::Stdout, rsn: i16, csn: i16, x: u16, y: u16) {
    let arrows : [&str;4] = ["▲","◀","▶","▼"];
    let h_arrow = if csn == 0 {
        "⚫"
    } else if csn > 0 {
        arrows[1]
    } else {
        arrows[2]
    };

    let v_arrow = if rsn == 0 {
        "⚫"
    } else if rsn > 0 {
        arrows[0]
    } else {
        arrows[3]
    };
    write!(stdout, "{c} {v}  {h}",
        c = cursor::Goto(x,y),
        v = v_arrow,
        h = h_arrow
    ).unwrap();
}

fn draw_window(stdout : &mut std::io::Stdout, state: &State) {
    // let (width, height) = termion::terminal_size().unwrap();
    write!(stdout,"{}{}",clear::All, cursor::Goto(1,1)).unwrap();
    draw_header(stdout, state);
    draw_body(stdout, state);
    if state.op != Operation::NoOp {
        draw_entry(stdout, state);
    }
    draw_save_state(stdout, state);
}

fn resize_data(data: &mut Vec<Vec<String>>, width: usize, height: usize) {
    for row in &mut data.iter_mut() { row.resize(width, "/".to_string()); }
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
            Key::Ctrl('r') => { // resize
                if state.op == Operation::NoOp {
                    state.op = Operation::Resizing;
                    state.resize_buffer.clear();
                } else if state.op == Operation::Resizing {
                    state.op = Operation::NoOp;
                    let params : Vec<&str> = state.resize_buffer.split(",").collect();
                    let nums : Vec<usize> = params.iter().map(|p| {p.parse::<usize>().unwrap()}).collect();
                    if nums.len() == 2 {
                        resize_data(&mut state.data, nums[0], nums[1]);
                        state.field = (0,0);
                    } else {
                        println!("RESIZE: Wrong number of parameters.");
                    }
                }
            }
            Key::Char(c) => {
                if c == '\n' {
                    match state.op {
                        Operation::Editing => {
                            state.op = Operation::NoOp;
                            let (x,y) = state.field;
                            state.data[y as usize][x as usize] = state.editor_buffer.to_string();
                            state.save_state = SaveState::Edited;
                        },
                        Operation::GoingTo => {
                            state.op = Operation::NoOp;
                             let params : Vec<&str> = state.goto_buffer.split(",").collect();
                             let nums : Vec<u32> = params.iter().map(|p| {p.parse::<u32>().unwrap()}).collect();
                             if nums.len() == 2 {
                                 state.field = (nums[0], nums[1]);
                             } else {
                                 println!("GOTO: Not enough parameters.");
                             }
                        },
                        Operation::Resizing => {
                            state.op = Operation::NoOp;
                            let params : Vec<&str> = state.resize_buffer.split(",").collect();
                            let nums : Vec<usize> = params.iter().map(|p| {p.parse::<usize>().unwrap()}).collect();
                            if nums.len() == 2 {
                                resize_data(&mut state.data, nums[0], nums[1]);
                                state.field = (0,0);
                            } else {
                                println!("RESIZE: Wrong number of parameters.");
                            }
                        },
                        _ => (),
                    }
                } else {
                    match state.op {
	                    Operation::Editing => state.editor_buffer.push(c),
	                    Operation::GoingTo => state.goto_buffer.push(c),
	                    Operation::Resizing => state.resize_buffer.push(c),
	                    _ => (),
                    }
                }
            },
            Key::Backspace => {
                match state.op {
                    Operation::Editing => {
                        state.editor_buffer.pop();
                        ()
                    },
                    Operation::GoingTo => {
                        state.goto_buffer.pop();
                        ()
                    },
                    Operation::Resizing => {
                        state.resize_buffer.pop();
                        ()
                    },
                    _ => (),
                }
            },
            Key::Ctrl('s') => {
                if state.save_state == SaveState::Edited {
                    coms::write(&state.data, Path::new(filename));
                    state.save_state = SaveState::Saved;
                }
            },
            Key::Left => {
                if state.field.0 > 0 && state.op == Operation::NoOp {
                    state.field.0 -= 1
                }
            },
            Key::Right => {
                if state.op == Operation::NoOp && state.field.0 < (state.data[0].len()-1) as u32 {
                    state.field.0 += 1
                }
            },
            Key::Up => {
                if state.field.1 > 0 && state.op == Operation::NoOp {
                    state.field.1 -= 1;
                }
            },
            Key::Down => {
                if state.op == Operation::NoOp && state.field.1 < (state.data.len()-1) as u32 {
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
