use std::ops::Add;

use fxhash::FxHashSet;
use rayon::prelude::*;

#[derive(Debug)]
pub enum State {
    Alive,
    Dead,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Point<const D: usize> {
    pub x: [i32; D],
}

impl<const D: usize> Point<D> {
    pub fn from_duple(x0: i32, x1: i32) -> Point<D> {
        let mut x = [0; D];
        x[0] = x0;
        x[1] = x1;
        Point { x }
    }

    /// Provides a list of all coordinates that are +/- 1 of the current point
    /// along any axis. For a `Point` with dimension `D` the list will always
    /// be of size `(3^D)-1`. The list does not include the centerpoint itself
    fn neighbors(&self) -> Vec<Point<D>> {
        let mut neighbors = Vec::new();
        let num_neighbors = 3i32.pow(D as u32)-1;
        for n in 0..=num_neighbors {
            let mut coords = [0;D];
            for (i, coord) in coords.iter_mut().enumerate() {
                // n as a base 3 number with D digits
                let pow = 3i32.pow(i as u32);
                let digit = (n / pow) % 3;
                // shift into the range [-1,1]
                let delta = digit-1;
                // apply delta to 
                *coord = self.x[i] + delta; 
            }
            if coords == self.x {
                continue;
            }
            let point = Point { x: coords };
            neighbors.push(point);
        }
        neighbors
    }

    fn neighbors_with_self(&self) -> Vec<Point<D>> {
        let mut neighbors = Vec::new();
        let num_neighbors = 3i32.pow(D as u32)-1;
        for n in 0..=num_neighbors {
            let mut coords = [0;D];
            for (i, coord) in coords.iter_mut().enumerate() {
                // n as a base 3 number with D digits
                let pow = 3i32.pow(i as u32);
                let digit = (n / pow) % 3;
                // shift into the range [-1,1]
                let delta = digit-1;
                // apply delta to 
                *coord = self.x[i] + delta; 
            }
            let point = Point { x: coords };
            neighbors.push(point);
        }
        neighbors
    }
}

impl<const D: usize> Add<Point<D>> for Point<D> {
    type Output = Point<D>;

    fn add(self, rhs: Point<D>) -> Self::Output {
        let mut out = [0; D];
        for (i, val) in out.iter_mut().enumerate() {
            *val = self.x[i] + rhs.x[i];
        }
        Point { x: out }
    }
}

pub struct Life<const D: usize> {
    alive: FxHashSet<Point<D>>,
    rule: fn(State, usize) -> State,
}

impl<const D: usize> Life<D> {
    pub fn new(initial: FxHashSet<Point<D>>, rule: fn(State, usize) -> State) -> Life<D> {
        Life { alive: initial, rule }
    }

    pub fn from_plate_default_rules(plate: &[&str]) -> Life<D> {
        Life::from_plate(plate, |state, neighbors| {
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

    pub fn from_plate(plate: &[&str], rule: fn(State, usize) -> State) -> Life<D> {
        let data: FxHashSet<Point<D>> = plate.iter().enumerate()
            .map(|(row, line)| {
                let chars: Vec<_> = line.char_indices()
                    .into_iter()
                    .map(|(col, chr)| (Point::from_duple(row as i32, col as i32), chr))
                    .collect();
                chars
            })
            .flatten()
            .filter_map(|(point, chr)| match chr {
                '#' => Some(point),
                _ => None,
            })
            .collect();
        Life::new(data, rule)
    }

    pub fn get(&self, point: &Point<D>) -> State {
        match self.alive.contains(point) {
            true => State::Alive,
            false => State::Dead,
        }
    }

    pub fn smart_bound(&self) -> FxHashSet<Point<D>> {
        self.alive.iter()
            .map(|point| point.neighbors_with_self())
            .flatten()
            .collect()
    }

    pub fn next(&self) -> Life<D> {
        let result: FxHashSet<Point<D>> = self.smart_bound().par_iter()
            .filter_map(|to_inspect| {
                let state = self.get(to_inspect);
                let neighbors = to_inspect.neighbors().iter()
                    .filter(|n| matches!(self.alive.get(n), Some(_)))
                    .count();
                match (self.rule)(state, neighbors) {
                    State::Alive => Some(to_inspect.clone()),
                    State::Dead => None,
                }
            })
            .collect();
        Life::new(result, self.rule)
        
    }

    pub fn active_cells(&self) -> usize {
        self.alive.len()
    }
}

