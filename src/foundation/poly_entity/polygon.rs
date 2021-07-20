use glam::Vec2;

struct Polygon {
    pub points: Vec<Vec2>,
}

struct PolygonIntersects {
    pub vertex: usize,
    pub intersects: Vec<usize>,
    pub points: Vec<usize>,
}

impl Polygon {
    pub fn clockwise(&self) -> bool {
        let mut sum = 0.;
        for i in 0..self.points.len() {
            let p0 = self.points[i];
            let p1 = self.points[(i + 1) % self.points.len()];

            sum += (p1.x - p0.x) * (p1.y + p0.y);
        }

        0. < sum
    }

    pub fn self_intersections(&self) -> Vec<PolygonIntersects> {}

    //remove self intersections and return 1 or more disconnected
    //non self intersecting polygons
    pub fn trim(&self) -> Vec<Self> {}

    pub fn inset(&self, amount: f32) -> Vec<Self> {}
}
