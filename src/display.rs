use std::fmt;

use crate::life::{Life, Point, State};


#[derive(Debug)]
pub enum Direction {
    Forward,
    Backwards,
}

struct Session<const D: usize> {
    game: Life<D>,
    position: Point<D>,
}

impl<const D: usize> Session<D> {
    pub fn update_position(&mut self, direction: Direction, dimension: usize) -> Result<Point<D>, String> {
        if dimension >= D {
            return Err(format!("Dimension '{}' is not present in this simulation", dimension));
        }
        let delta = match direction {
            Direction::Forward => 1,
            Direction::Backwards => -1,
        };
        self.position.x[dimension] += delta;
        Ok(self.position.clone())
    }
}

impl<const D: usize> fmt::Display for Session<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((w, h)) = term_size::dimensions() {
            let (w, h) = (w as i32, h as i32);
            let (x_sc , y_sc) = ((w/2) as i32, (h/2) as i32);
            let center = &self.position;
            let mut screen: String =  (0..h).map(|y| {
                let line: String = (0..w).map(|x: i32| {
                    let point = Point::from_duple(center.x[0] - x_sc + x, center.x[1] - y_sc + y);
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