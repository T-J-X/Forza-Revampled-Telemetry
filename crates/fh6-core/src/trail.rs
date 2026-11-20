pub const MAX_NODES: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub z: f32,
}

pub struct WorldMapTrailBuffer {
    nodes: [Point; MAX_NODES],
    head: usize,
    count: usize,
    last_x: f32,
    last_z: f32,
    has_last: bool,
}

impl WorldMapTrailBuffer {
    pub fn new() -> Self {
        Self {
            nodes: [Point { x: 0.0, z: 0.0 }; MAX_NODES],
            head: 0,
            count: 0,
            last_x: 0.0,
            last_z: 0.0,
            has_last: false,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn reset(&mut self) {
        self.head = 0;
        self.count = 0;
        self.has_last = false;
    }

    pub fn update(&mut self, pos_x: f32, pos_z: f32) {
        if !pos_x.is_finite() || !pos_z.is_finite() {
            return;
        }

        if !self.has_last {
            self.last_x = pos_x;
            self.last_z = pos_z;
            self.has_last = true;
            self.nodes[self.head] = Point { x: pos_x, z: pos_z };
            self.head = (self.head + 1) % MAX_NODES;
            self.count = 1;
            return;
        }

        let dx = pos_x - self.last_x;
        let dz = pos_z - self.last_z;
        let dist_sq = dx * dx + dz * dz;

        if dist_sq > 10000.0 {
            self.reset();
            self.last_x = pos_x;
            self.last_z = pos_z;
            self.has_last = true;
            self.nodes[self.head] = Point { x: pos_x, z: pos_z };
            self.head = (self.head + 1) % MAX_NODES;
            self.count = 1;
            return;
        }

        if dist_sq >= 4.0 {
            self.last_x = pos_x;
            self.last_z = pos_z;
            self.nodes[self.head] = Point { x: pos_x, z: pos_z };
            self.head = (self.head + 1) % MAX_NODES;
            if self.count < MAX_NODES {
                self.count += 1;
            }
        }
    }

    pub fn get_history_nodes(&self) -> Vec<Point> {
        let mut out = Vec::with_capacity(self.count);
        let start = if self.count < MAX_NODES { 0 } else { self.head };
        for i in 0..self.count {
            let idx = (start + i) % MAX_NODES;
            out.push(self.nodes[idx]);
        }
        out
    }

    pub fn populate_procedural_touge(&mut self, time_sec: f64, speed_ms: f32, yaw_curv: f32) {
        self.reset();
        self.has_last = true;
        let dt: f32 = 0.05;
        let dt_f64 = dt as f64;
        let mut sim_x: f32 = 0.0;
        let mut sim_z: f32 = 0.0;
        let mut heading: f32 = 0.0;
        for i in 0..MAX_NODES {
            let t = time_sec - (MAX_NODES as f64) * dt_f64 + (i as f64) * dt_f64;
            let curv = yaw_curv * (t * 1.4).sin() as f32;
            heading += curv * speed_ms * dt;
            sim_x += heading.sin() * speed_ms * dt;
            sim_z += heading.cos() * speed_ms * dt;
            self.nodes[self.head] = Point { x: sim_x, z: sim_z };
            self.head = (self.head + 1) % MAX_NODES;
            self.count += 1;
        }
        self.last_x = sim_x;
        self.last_z = sim_z;
    }
}

impl Default for WorldMapTrailBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_update_sets_one_node() {
        let mut b = WorldMapTrailBuffer::new();
        b.update(1.0, 2.0);
        assert_eq!(b.count(), 1);
        let h = b.get_history_nodes();
        assert_eq!(h[0], Point { x: 1.0, z: 2.0 });
    }

    #[test]
    fn distance_threshold() {
        let mut b = WorldMapTrailBuffer::new();
        b.update(0.0, 0.0);
        b.update(1.0, 0.0); // dist 1 <2 -> no add
        assert_eq!(b.count(), 1);
        b.update(2.1, 0.0); // dist 2.1 >2 -> add
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn teleport_resets() {
        let mut b = WorldMapTrailBuffer::new();
        b.update(0.0, 0.0);
        b.update(10.0, 0.0);
        b.update(200.0, 0.0); // dist >100 -> reset
        assert_eq!(b.count(), 1);
        let h = b.get_history_nodes();
        assert_eq!(h[0].x, 200.0);
    }

    #[test]
    fn ring_wraps() {
        let mut b = WorldMapTrailBuffer::new();
        for i in 0..130 {
            b.update(i as f32 * 3.0, 0.0);
        }
        assert_eq!(b.count(), MAX_NODES);
        let h = b.get_history_nodes();
        assert_eq!(h[0].x, (130 - MAX_NODES) as f32 * 3.0);
    }

    #[test]
    fn procedural_populates_120() {
        let mut b = WorldMapTrailBuffer::new();
        b.populate_procedural_touge(10.0, 30.0, 0.5);
        assert_eq!(b.count(), MAX_NODES);
        let h = b.get_history_nodes();
        assert_eq!(h.len(), MAX_NODES);
    }
}
