use std::{fmt, process, thread};
use std::time::Duration;

use crossbeam_channel::bounded;
use ncurses;

use crate::life::{Life, Point, State};

#[derive(Debug)]
pub enum Direction {
    Forward,
    Backwards,
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
}

impl<const D: usize> Session<D> {
    fn update_position(&mut self, direction: Direction, dimension: usize) -> Result<Point<D>, String> {
        if dimension >= D {
            return Err(format!("Dimension '{}' is not present in this simulation", dimension));
        }
        let delta = match direction {
            Direction::Forward => 1,
            Direction::Backwards => -1,
        };
        self.screen_center.x[dimension] += delta;
        Ok(self.screen_center.clone())
    }

    fn step(&mut self, rule: fn(State, usize) -> State) {
        self.game = self.game.next(rule)
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
                line + "\n"
            })
            .collect();
            screen.pop();
            return write!(f, "{}", screen);
        }
        write!(f, "failed render the board")
    }
}

fn animate<const D: usize>(game: Life<D>, center: Point<D>, rule: fn(State, usize) -> State) {
    // set up display
    ncurses::initscr();
    ncurses::noecho();

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
    let mut session = Session {
        game,
        screen_center: center,
    };

    // animation loop
    loop {
        match r.recv_timeout(Duration::from_millis(10)) {
            Ok(0) => graceful_exit(),
            Ok(val) => handle_input(val, &mut session),
            Err(_) => { /* leave the screen alone */ },
        };
        thread::sleep(Duration::from_millis(100));
    }
}

fn graceful_exit() {
    ncurses::endwin();
    println!("Program exited");
    process::exit(0);
}

fn handle_input<const D: usize>(val: i32, session: &mut Session<D>) {
    ncurses::clear();
    let output = match std::char::from_u32(val as u32) {
        Some(chr) => format!("Intercepted Character: '{}'\n", chr),
        None => format!("Intercepted Value: '{}'\n", val),
    };
    ncurses::addstr(&output);
}