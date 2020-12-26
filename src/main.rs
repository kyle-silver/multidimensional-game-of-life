#![feature(min_const_generics)]

mod life;
mod display;

use life::{Life, Point};

extern crate ncurses;

fn main()
{
    let plate: Vec<&str> = include_str!("../res/gosper-glider-gun.txt").split('\n').collect();
    let game: Life<2> = Life::from_plate_default_rules(&plate);

    display::animate(game, Point::from_duple(20, 5))
}