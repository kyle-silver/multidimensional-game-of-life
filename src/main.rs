#![feature(min_const_generics)]

mod life;
mod display;

use life::{Life, Point, State};

extern crate ncurses;

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
}