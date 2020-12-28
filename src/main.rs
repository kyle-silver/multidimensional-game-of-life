#![feature(min_const_generics)]

mod life;
mod display;

use display::Playback;
use life::{Life, Point};


fn main()
{
    let plate: Vec<&str> = include_str!("../res/gosper-glider-gun.txt").split('\n').collect();
    let game: Life<2> = Life::from_plate(&plate, life::STANDARD_RULES);

    display::animate(game, Point::from_duple(20, 5), Playback::Step);
}