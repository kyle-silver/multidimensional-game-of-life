use std::{fmt, process, thread};
use std::time::Duration;

use crossbeam_channel::bounded;
use ncurses::{self, FALSE};

use crate::life::{Life, Point, State};

#[derive(Debug)]
enum Direction {
    Forward,
    Backwards,
}

enum UserInput {
    Up,
    Down,
    Left,
    Right,
    Move { dir: Direction, axis: usize, },
    Pause,
    Step,
    Exit,
    Noop,
}

impl UserInput {
    fn new(input: i32) -> UserInput {
        if let Some(chr) = std::char::from_u32(input as u32).map(|c| c.to_ascii_lowercase()) {
            use UserInput::*;
            return match chr {
                'q' => Exit,
                'w' => Up,
                'a' => Left,
                's' => Down,
                'd' => Right,
                'h' => Left,
                'j' => Down,
                'k' => Up,
                'l' => Right,
                ' ' => Pause,
                '\n' => Step,
                _ => Noop,
            };
        }
        UserInput::Noop
    }
}

struct ScreenDimensions {
    width: i32,
    height: i32,
    radius_w: i32,
    radius_h: i32,
}

impl ScreenDimensions {
    fn current() -> Option<ScreenDimensions> {
        if let Some((w, h)) = term_size::dimensions() {
            let (w, h) = (w as i32, h as i32);
            let sd = ScreenDimensions {
                width: w,
                height: h,
                radius_w: w/2,
                radius_h: h/2,
            };
            return Some(sd);
        }
        None
    }

    fn screen_coordinate_to_board_position<const D: usize>(&self, screen_position: (i32, i32), board_center: &Point<D>) -> Point<D> {
        let (w, h) = screen_position;
        let grid_x = board_center.x[0] - self.radius_w + w;
        let grid_y = board_center.x[1] - self.radius_h + h;
        Point::from_duple(grid_x, grid_y)
    }
}

struct Session<const D: usize> {
    game: Life<D>,
    screen_center: Point<D>,
    paused: bool,
}

impl<const D: usize> Session<D> {
    fn new(game: Life<D>, screen_center: Point<D>) -> Session<D> {
        Session {
            game,
            screen_center,
            paused: false,
        }
    }

    fn handle(&mut self, input: UserInput) {
        match input {
            UserInput::Up => {
                self.update_position(Direction::Backwards, 1);
            },
            UserInput::Down => {
                self.update_position(Direction::Forward, 1);
            },
            UserInput::Left => {
                self.update_position(Direction::Backwards, 0);
            },
            UserInput::Right => {
                self.update_position(Direction::Forward, 0);
            },
            UserInput::Move { dir, axis } => {
                self.update_position(dir, axis);
            },
            UserInput::Pause => {
                self.paused = !self.paused;
            },
            UserInput::Step => {
                if self.paused {
                    self.step();
                } 
            }
            UserInput::Exit => {
                graceful_exit()
            },
            UserInput::Noop => {},
        };
    }

    fn update_position(&mut self, direction: Direction, dimension: usize) {
        if dimension >= D {
            return;
        }
        let delta = match direction {
            Direction::Forward => 1,
            Direction::Backwards => -1,
        };
        self.screen_center.x[dimension] += delta;
    }

    fn step(&mut self) {
        self.game = self.game.next()
    }
}

impl<const D: usize> fmt::Display for Session<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(dimensions) = ScreenDimensions::current() {
            let center = &self.screen_center;
            let mut screen: String = (0..dimensions.height).map(|y| {
                let line: String = (0..dimensions.width).map(|x| {
                    let point = dimensions.screen_coordinate_to_board_position((x, y), center);
                    match self.game.get(&point) {
                        State::Alive => '#',
                        State::Dead => ' ',
                    }
                })
                .collect();
                line
            })
            .collect();
            screen.pop();
            return write!(f, "{}", screen);
        }
        write!(f, "failed render the board")
    }
}

pub fn animate<const D: usize>(game: Life<D>, center: Point<D>) {
    // set up display
    ncurses::initscr();
    ncurses::noecho();
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    // start listener for user input
    let (s, r) = bounded(1);
    thread::spawn(move || {
        // try send so we don't spam the buffer with repeated inputs
        // getch is also a blocking operation so we aren't wasting CPU cycles
        loop {
            s.try_send(ncurses::getch()).unwrap_or(());
        }
    });

    // session data
    let mut session = Session::new(game, center);

    // draw first image
    draw(&session);

    // animation loop
    loop {
        match r.recv_timeout(Duration::from_millis(10)) {
            Ok(0) => graceful_exit(),
            Ok(val) => handle_input(val, &mut session),
            Err(_) => { /* leave the screen alone */ },
        };
        if !session.paused {
            session.step();
        }
        draw(&session);
        thread::sleep(Duration::from_millis(100));
    }
}

fn graceful_exit() {
    ncurses::endwin();
    println!("Program exited");
    process::exit(0);
}

fn handle_input<const D: usize>(val: i32, session: &mut Session<D>) {
    let input = UserInput::new(val);
    session.handle(input);
}

fn draw<const D: usize>(session: &Session<D>) {
    ncurses::clear();
    let mut screen = format!(
        "{}\rCtrl+C to Exit | x: {:?} | Live Cells: {}", 
        session, 
        session.screen_center.x, 
        session.game.active_cells()
    );
    if session.paused {
        screen += " | paused";
    }
    ncurses::addstr(&screen);
    ncurses::refresh();
    
}