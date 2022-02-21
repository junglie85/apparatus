use core::iter::Iterator;

pub struct BresenhamLine {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    error: i32,
    line_complete: bool,
}

impl BresenhamLine {
    pub fn new(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        let (x0, y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();

        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let error = dx + dy;

        Self {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            error,
            line_complete: false,
        }
    }
}

/// Compute the next (x, y) coordinate pair for the given line.
/// See https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm for details.
impl Iterator for BresenhamLine {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x0 as u32, self.y0 as u32);

        if self.line_complete {
            return None;
        }

        if self.x0 == self.x1 && self.y0 == self.y1 {
            self.line_complete = true;
        }

        let error_2 = 2 * self.error;
        if error_2 >= self.dy {
            if self.x0 == self.x1 {
                self.line_complete = true;
            } else {
                self.error += self.dy;
                self.x0 += self.sx;
            }
        }
        if error_2 < self.dx {
            if self.y0 == self.y1 {
                self.line_complete = true;
            } else {
                self.error += self.dx;
                self.y0 += self.sy;
            }
        }

        Some(point)
    }
}
