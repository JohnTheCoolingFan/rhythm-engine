use lyon_geom::CubicBezierSegment;
use lyon_geom::Point;

use crate::utils::from_end::FromEnd;

pub struct FreeAnchor(Vec<Point<f32>>, f32, Vec<Point<f32>>, Vec<f32>);
struct CSplineCache {
    seg_num: usize,
    point_num: usize,
}

pub struct ComplexSpline {
    //cubic splines
    //x: anchor
    //0: arm
    //Vec pattern: x00x00x00x
    //arms are relative to anchors
    points: Vec<Point<f32>>,

    scale: f32,
    rotation: f32,

    //offsets relative to poly entity spline collection is attached to
    anchor_offsets: Vec<f32>,
    segment_approximations: Vec<Vec<Point<f32>>>,
    approximation_displacements: Vec<Vec<f32>>,

    cache: CSplineCache,
}

impl ComplexSpline {
    pub fn resample_segment(&mut self, seg_num: usize) {
        assert!(seg_num * 3 + 3 < self.points.len());

        let i = seg_num * 3;
        let p0 = self.points[i];
        let p1 = self.points[i + 1].to_vector() + p0.to_vector();
        let p3 = self.points[i + 3];
        let p2 = self.points[i + 2].to_vector() + p3.to_vector();

        let segment = CubicBezierSegment::<f32> {
            from: p0,
            ctrl1: Point::new(p1.x, p1.y),
            ctrl2: Point::new(p2.x, p2.y),
            to: p3,
        };

        let aprox = &mut self.segment_approximations[seg_num];
        let displ = &mut self.approximation_displacements[seg_num];

        aprox.clear();
        aprox.push(self.points[seg_num * 3]);
        displ.clear();
        displ.push(0.0);

        segment.for_each_flattened(0.05, &mut |p| {
            displ
                .push((p.to_vector() - aprox[FromEnd(0)].to_vector()).length() + displ[FromEnd(0)]);
            aprox.push(p);
        });
    }

    pub fn push_anchor(&mut self, x: f32, y: f32, offset: f32) {
        self.points
            .extend(&[Point::new(0.0, 0.0), Point::new(0.0, 0.0), Point::new(x, y)]);
        self.anchor_offsets.push(offset);
        self.segment_approximations.push(vec![]);
        self.approximation_displacements.push(vec![]);
        self.resample_segment(self.anchor_offsets.len() - 2);
    }

    pub fn pop_anchor(&mut self) -> Option<FreeAnchor> {
        if self.anchor_offsets.len() < 3 {
            None
        } else {
            Some(FreeAnchor(
                self.points.split_off(self.points.len() - 3),
                self.anchor_offsets.pop().unwrap(),
                self.segment_approximations.pop().unwrap(),
                self.approximation_displacements.pop().unwrap(),
            ))
        }
    }

    pub fn insert_anchor(&mut self, x: f32, y: f32, offset: f32, index: usize) {
        if index == self.anchor_offsets.len() {
            self.push_anchor(x, y, offset);
        } else {
            assert!(index != 0 && index < self.anchor_offsets.len());
            self.points = [
                &self.points[..(index * 3 - 1)],
                &[Point::new(0.0, 0.0), Point::new(x, y), Point::new(0.0, 0.0)],
                &self.points[(index * 3 - 1)..],
            ]
            .concat();

            self.anchor_offsets.insert(index, offset);
            self.segment_approximations.insert(index - 1, vec![]);
            self.resample_segment(index - 1);
            self.resample_segment(index);
        }
    }

    pub fn remove_anchor(&mut self, index: usize) -> Option<FreeAnchor> {
        let remove_point = index * 3;
        if remove_point == self.anchor_offsets.len() - 1 {
            self.pop_anchor()
        } else {
            None
        }
    }

    fn depth_2_search(&self, offset: f32, err_index: usize) -> Point<f32> {
        let seg_num = err_index - 1;
        let seg_rel_s = (offset - self.anchor_offsets[seg_num])
            / (self.anchor_offsets[seg_num + 1] - self.anchor_offsets[seg_num]);

        let displ = &(self.approximation_displacements[seg_num]);
        let aprox = &(self.segment_approximations[seg_num]);

        match displ.binary_search_by(|s| s.partial_cmp(&seg_rel_s).unwrap()) {
            Ok(point) => aprox[point],
            Err(aprx_err_index) => match aprx_err_index {
                l if l == aprox.len() => aprox[FromEnd(0)],
                0 => aprox[0],
                _ => {
                    let p = aprx_err_index;
                    let lerp_amount = (seg_rel_s - displ[err_index - 1])
                        / (displ[err_index] - displ[err_index - 1]);
                    aprox[p - 1].lerp(aprox[p], lerp_amount)
                }
            },
        }
    }

    fn depth_1_search(&self, offset: f32) -> Point<f32> {
        match self
            .anchor_offsets
            .binary_search_by(|t| t.partial_cmp(&offset).unwrap())
        {
            Ok(anchor_num) => self.points[anchor_num * 3],
            Err(err_index) => self.depth_2_search(offset, err_index),
        }
    }

    fn depth_0_search(&self, offset: f32) -> Point<f32> {
        if offset <= self.anchor_offsets[0] {
            self.points[0]
        } else if self.anchor_offsets[FromEnd(0)] <= offset {
            self.points[FromEnd(0)]
        } else {
            self.depth_1_search(offset)
        }
    }

    pub fn interp(&self, offset: f32) -> Point<f32> {
        self.depth_0_search(offset)
    }
}
