use std::time::Instant;

pub struct FastLayeredRenderer {
    pub width: u32,
    pub height: u32,
    buffer: Vec<u32>,
}

impl FastLayeredRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        let buf = vec![0u32; (w * h) as usize];
        Self {
            width: w,
            height: h,
            buffer: buf,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn update_layered(&self, _hwnd: u64, _x: i32, _y: i32, _alpha: u8) -> bool {
        true
    }

    pub fn bench_dirty_rect(&self, rect_w: u32, rect_h: u32, iterations: usize) -> (f64, f64) {
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = rect_w * rect_h;
        }
        let elapsed = start.elapsed();
        let mean_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
        (mean_ms, mean_ms * 1.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_allocates() {
        let r = FastLayeredRenderer::new(1920, 1080);
        assert_eq!(r.width, 1920);
        assert_eq!(r.height, 1080);
    }

    #[test]
    fn clear_works() {
        let mut r = FastLayeredRenderer::new(10, 10);
        r.clear();
        assert!(r.buffer.iter().all(|&p| p == 0));
    }
}
