//! Connected areas of a pixel mask, with small gaps closed, so a row of LED caps or the strokes
//! of one label count as one area.
use crate::geom::Rect;

/// `mask` grown by `r` pixels in every direction (a square dilation).
pub fn dilate(mask: &[bool], w: u32, h: u32, r: u32) -> Vec<bool> {
    let (wu, hu, ru) = (w as usize, h as usize, r as usize);
    let pass = |src: &[bool], horizontal: bool| -> Vec<bool> {
        let (lines, len) = if horizontal { (hu, wu) } else { (wu, hu) };
        let mut out = vec![false; src.len()];
        let at = |line: usize, i: usize| if horizontal { line * wu + i } else { i * wu + line };
        for line in 0..lines {
            // Trues in the window [i - r, i + r], kept as a running count.
            let mut count = 0usize;
            for i in 0..len.min(ru + 1) {
                count += usize::from(src.get(at(line, i)).copied().unwrap_or(false));
            }
            for i in 0..len {
                if let Some(o) = out.get_mut(at(line, i)) {
                    *o = count > 0;
                }
                let enter = i + ru + 1;
                if enter < len {
                    count += usize::from(src.get(at(line, enter)).copied().unwrap_or(false));
                }
                if i >= ru {
                    count -= usize::from(src.get(at(line, i - ru)).copied().unwrap_or(false));
                }
            }
        }
        out
    };
    pass(&pass(mask, true), false)
}

/// The area each `true` pixel of `mask` belongs to (8-connected after closing gaps of up to
/// `gap` px), as an index per pixel (`u32::MAX` off the mask), and how many areas there are.
pub fn label(mask: &[bool], w: u32, h: u32, gap: u32) -> (Vec<u32>, usize) {
    let grown = dilate(mask, w, h, gap / 2);
    let (wu, wi, hi) = (w as usize, i64::from(w), i64::from(h));
    let mut ids = vec![u32::MAX; grown.len()];
    let mut count = 0usize;
    let mut stack = Vec::new();
    for start in 0..grown.len() {
        if !grown[start] || ids[start] != u32::MAX {
            continue;
        }
        let id = u32::try_from(count).unwrap_or(u32::MAX);
        count += 1;
        ids[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = ((i % wu) as i64, (i / wu) as i64);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= wi || ny >= hi {
                        continue;
                    }
                    let j = ny as usize * wu + nx as usize;
                    if grown[j] && ids[j] == u32::MAX {
                        ids[j] = id;
                        stack.push(j);
                    }
                }
            }
        }
    }
    // Only the mask's own pixels keep a label; the grown margin was for joining.
    for (id, m) in ids.iter_mut().zip(mask) {
        if !m {
            *id = u32::MAX;
        }
    }
    (ids, count)
}

/// A box that grows to hold points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            x0: u32::MAX,
            y0: u32::MAX,
            x1: 0,
            y1: 0,
        }
    }
}

impl Bounds {
    /// Takes in `(x, y)`.
    pub fn add(&mut self, x: u32, y: u32) {
        (self.x0, self.y0, self.x1, self.y1) = (self.x0.min(x), self.y0.min(y), self.x1.max(x), self.y1.max(y));
    }

    /// The box, or `None` when no point was added.
    pub fn rect(&self) -> Option<Rect> {
        (self.x0 <= self.x1 && self.y0 <= self.y1).then(|| {
            Rect::new(
                i64::from(self.x0),
                i64::from(self.y0),
                i64::from(self.x1 - self.x0 + 1),
                i64::from(self.y1 - self.y0 + 1),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_pieces_join_and_far_ones_do_not() {
        // Three 2x2 blocks: two 4 px apart, one 20 px away.
        let (w, h) = (40u32, 6u32);
        let mut mask = vec![false; (w * h) as usize];
        for (x0, y0) in [(2u32, 2u32), (8, 2), (30, 2)] {
            for y in y0..y0 + 2 {
                for x in x0..x0 + 2 {
                    mask[(y * w + x) as usize] = true;
                }
            }
        }
        let (ids, count) = label(&mask, w, h, 6);
        assert_eq!(count, 2);
        assert_eq!(ids[(2 * w + 2) as usize], ids[(2 * w + 8) as usize]);
        assert_ne!(ids[(2 * w + 2) as usize], ids[(2 * w + 30) as usize]);
        assert_eq!(ids[0], u32::MAX);
        let mut b = Bounds::default();
        assert!(b.rect().is_none());
        b.add(3, 4);
        b.add(1, 2);
        assert_eq!(b.rect(), Some(Rect::new(1, 2, 3, 3)));
    }
}
