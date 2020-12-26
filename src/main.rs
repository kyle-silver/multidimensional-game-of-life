#![feature(min_const_generics)]

mod life;
mod display;

use life::{Life, Point, State};

extern crate ncurses;

use std::{process, thread};
use std::time::Duration;

use crossbeam_channel::bounded;
// use ncurses::*;

fn main()
{
    let plate: Vec<&str> = include_str!("../res/gosper-glider-gun.txt").split('\n').collect();
    let game: Life<2> = Life::from_plate(&plate);

    display::animate(game, Point::from_duple(20, 5),|state, neighbors| {
        match state {
            State::Alive => {
                if neighbors == 2 || neighbors == 3 {
                    State::Alive
                } else {
                    State::Dead
                }
            },
            State::Dead => {
                if neighbors == 3 {
                    State::Alive
                } else {
                    State::Dead
                }
            }
        }
    })

    // ncurses::initscr();
    // ncurses::noecho();

    // let (s, r) = bounded(1);

    // thread::spawn(move || {
    //     loop {
    //         // try send so we don't spam the buffer with repeated inputs
    //         s.try_send(ncurses::getch()).unwrap_or(());
    //     }
    // });

    // for i in 0..10_000 {
    //     match r.recv_timeout(Duration::from_millis(10)) {
    //         Ok(0) => {
    //             // gracefully exit
    //             ncurses::endwin();
    //             process::exit(0);
    //         }
    //         Ok(val) => {
    //             // update the screen
    //             ncurses::clear();
    //             let output = match std::char::from_u32(val as u32) {
    //                 Some(chr) => format!("Intercepted Character: '{}'\n", val),
    //                 None => format!("Intercepted Value: '{}'\n", val),
    //             };
    //             ncurses::addstr(&output);
    //         }
    //         Err(_) => { /* leave the screen alone */ }
    //     };
    //     ncurses::addstr(&format!("\rFame Number: {}", i));
    //     ncurses::refresh();
    //     thread::sleep(Duration::from_millis(100));
    // }
}