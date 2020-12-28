use std::{fmt, process, thread};
use std::time::Duration;

use crossbeam_channel::bounded;
use ncurses::*;

use crate::life::{Life, Point, State};

#[derive(Debug)]
enum Direction {
    Forward,
    Backwards,
}

pub enum Playback {
    Play,
    Step
}

enum UserInput {
    Up,
    Down,
    Left,
    Right,
    Move(Direction, usize),
    Pause,
    Step,
    Exit,
    Noop,
}

impl UserInput {
    fn new(input: i32) -> UserInput {
        if let Some(chr) = std::char::from_u32(input as u32).map(|c| c.to_ascii_lowercase()) {
            use UserInput::*;
            use Direction::*;
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
                '1' => Move(Forward, 0),
                '!' => Move(Backwards, 0),
                '2' => Move(Backwards, 1),
                '@' => Move(Forward, 1),
                '3' => Move(Backwards, 2),
                '#' => Move(Forward, 2),
                '4' => Move(Backwards, 3),
                '$' => Move(Forward, 3),
                '5' => Move(Backwards, 4),
                '%' => Move(Forward, 4),
                '6' => Move(Backwards, 5),
                '^' => Move(Forward, 5),
                '7' => Move(Backwards, 6),
                '&' => Move(Forward, 6),
                '8' => Move(Backwards, 7),
                '*' => Move(Forward, 7),
                '9' => Move(Backwards, 8),
                '(' => Move(Forward, 8),
                '0' => Move(Backwards, 9),
                ')' => Move(Forward, 9),
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

    fn screen_coordinate_to_board_position<const D: usize>(
        &self, 
        screen_position: (i32, i32), 
        board_center: &Point<D>
    ) -> Point<D> {
        let (w, h) = screen_position;
        let grid_x = board_center.x[0] - self.radius_w + w;
        let grid_y = board_center.x[1] - self.radius_h + h;
        let mut point = board_center.clone();
        point.x[0] = grid_x;
        point.x[1] = grid_y;
        point
    }
}

struct Session<const D: usize> {
    game: Life<D>,
    screen_center: Point<D>,
    playback: Playback,
}

impl<const D: usize> Session<D> {
    fn new(game: Life<D>, screen_center: Point<D>, playback: Playback) -> Session<D> {
        Session { game, screen_center, playback, }
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
            UserInput::Move(dir, dim) => {
                self.update_position(dir, dim);
            },
            UserInput::Pause => {
                self.playback = match self.playback {
                    Playback::Play => Playback::Step,
                    Playback::Step => Playback::Play,
                };
            },
            UserInput::Step => {
                match self.playback {
                    Playback::Play => {}
                    Playback::Step => {
                        self.step();
                    },
                };
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

pub fn animate<const D: usize>(game: Life<D>, screen_center: Point<D>, playback: Playback) {
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
    let mut session = Session::new(game, screen_center, playback);

    // draw first image
    draw(&session);

    // animation loop
    loop {
        let repaint: bool = match r.recv_timeout(Duration::from_millis(10)) {
            Ok(0) => {
                graceful_exit();
                false
            },
            Ok(val) => {
                handle_input(val, &mut session);
                true
            },
            Err(_) => { if matches!(session.playback, Playback::Play) { true } else { false } },
        };
        if matches!(session.playback, Playback::Play) {
            session.step();
        }
        if repaint {
            draw(&session);
        }
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
    if matches!(session.playback, Playback::Step) {
        screen += " | paused";
    }
    ncurses::addstr(&screen);
    ncurses::refresh();
    
}