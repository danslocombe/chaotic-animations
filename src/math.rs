extern crate rand;
use self::rand::distributions::{IndependentSample, Range};
use cgmath::{Point3, Vector3};
use std::f64::consts::PI;

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

#[derive(Debug, Clone)]
pub struct VFParams {
    a1: usize,
    a2: usize,
    a3: usize,
    p: f64,
    q: f64,
    r: f64,
    s: f64,
    v: f64,
    w: f64,
    u: f64,
}

impl VFParams {
    pub fn gen_rand() -> Self {
        let mut rng = rand::thread_rng();
        let dimension_range: Range<usize> = Range::new(0, 2);
        let param_range: Range<f64> = Range::new(-1.0, 1.0);
        VFParams {
            a1: dimension_range.ind_sample(&mut rng),
            a2: dimension_range.ind_sample(&mut rng),
            a3: dimension_range.ind_sample(&mut rng),
            p: param_range.ind_sample(&mut rng),
            q: param_range.ind_sample(&mut rng),
            r: param_range.ind_sample(&mut rng),
            s: param_range.ind_sample(&mut rng),
            v: param_range.ind_sample(&mut rng),
            w: param_range.ind_sample(&mut rng),
            u: param_range.ind_sample(&mut rng),
        }
    }

    pub fn to_string(&self) -> String {
        format!("a1: {}, a2: {}, a3: {}, p: {:.3}, q: {:.3}, r: {:.3}, s: \
                 {:.3}, v: {:.3}, w: {:.3}, u: {:.3}",
                self.a1,
                self.a2,
                self.a3,
                self.p,
                self.q,
                self.r,
                self.s,
                self.v,
                self.w,
                self.u)
    }
}

pub fn mutate_point(dt: f64, p: &Point, params: &VFParams) -> Point {
    let Point { x: i, y: j, z: k } = vector_field(p, params);
    Point::new(p.x + dt * i, p.y + dt * j, p.z + dt * k)
}

fn vector_field(p: &Point, params: &VFParams) -> Point {
    let Point { x, y, z } = *p;

    let shuffle = vec![x, y, z];

    let i = params.q *
            (shuffle[params.a3] /
             (shuffle[params.a2] +
              params.p * (shuffle[params.a1] + params.q) * PI +
              z) * PI)
        .cos();
    let j = params.r *
            (shuffle[params.a2] * (PI * x).cos() /
             (shuffle[params.a2] + params.r * -(x + params.s)) *
             PI + x)
        .sin();
    let k = params.w *
            (shuffle[params.a1] /
             (shuffle[params.a2] +
              params.v / (shuffle[params.a3] - shuffle[params.a2] + params.w)) *
             PI + shuffle[params.a1])
        .sin();

    Point::new(i, j, k)
}
