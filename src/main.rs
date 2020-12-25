#![feature(min_const_generics)]

mod life;
mod display;

extern crate ncurses;

use std::{process, thread};
use std::time::Duration;

use crossbeam_channel::bounded;
use ncurses::*;

fn main()
{
    initscr();
    noecho();

    let (s, r) = bounded(1);

    thread::spawn(move || {
        loop {
            s.try_send(getch()).unwrap_or(());
        }
    });

    for i in 0..10_000 {
        match r.recv_timeout(Duration::from_millis(10)) {
            Ok(0) => {
                endwin();
                process::exit(0);
            }
            Ok(val) => {
                clear();
                let output = match std::char::from_u32(val as u32) {
                    Some(chr) => format!("Intercepted Character: '{}'\n", chr),
                    None => format!("Intercepted Value: '{}'\n", val),
                };
                addstr(&output);
                // refresh();
            }
            Err(_) => { /* leave the screen alone */}
        };
        addstr(&format!("\rFame Number: {}", i));
        refresh();
        thread::sleep(Duration::from_millis(100));
    }
}