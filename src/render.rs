
use cgmath::{Matrix4, Point2, Vector4};
use graphics::math::hsv;
use graphics::types::Color;
use math::*;
use opengl_graphics::GlGraphics;
use piston::input::*;
use std::f64::consts::PI;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

#[derive (Debug)]
pub struct Camera {
    pub pos: Point,
    pub dir: Vector,
    pub scale: f64,
    pub proj: CameraProjection,
}

#[derive (Debug)]
pub enum CameraProjection {
    Orthographic,
    Perspective,
}

impl Camera {
    pub fn update(&mut self, time: f64) {
        use CameraProjection::*;
        if let Perspective = self.proj {
            // Orbit
            // TODO move consts
            let r = 10.0;
            let timemult = 0.004;
            self.pos = Point {
                z: r * (timemult * time).sin(),
                x: r * (timemult * time).cos(),
                y: 0.5,
            };
        }
    }
}

pub enum PlotStyle {
    Point,
    Line,
    Radial,
}

pub fn plot_points(gl: &mut GlGraphics,
                   camera: &Camera,
                   args: &RenderArgs,
                   plot_style: &PlotStyle,
                   width: f64,
                   wavefront: &[Point],
                   wavefront_prev: &[Point]) {
    //  Assume wavefront length = wavefront_prev length
    for (i, p) in wavefront.iter().enumerate() {
        let color = gen_color(i as f64 / wavefront.len() as f64);
        match *plot_style {
            PlotStyle::Point => {
                line_points(gl,
                            camera,
                            args,
                            color,
                            width,
                            p,
                            &wavefront_prev[i]);
            }
            PlotStyle::Line => {
                if i > 0 {
                    line_points(gl,
                                camera,
                                args,
                                color,
                                width,
                                p,
                                &wavefront[i - 1]);
                }
            }
            PlotStyle::Radial => {
                line_points(gl,
                            camera,
                            args,
                            color,
                            width,
                            p,
                            &Point::new(0.0, 0.0, 0.0));
            }
        }
    }
}

fn gen_color(i: f64) -> Color {
    hsv(RED, (i * 2.0 * PI) as f32, 0.8, 1.0)
}


fn line_points(gl: &mut GlGraphics,
               camera: &Camera,
               args: &RenderArgs,
               color: Color,
               width: f64,
               p1: &Point,
               p2: &Point) {
    use graphics::*;

    let (x_mid, y_mid) = ((args.width / 2) as f64, (args.height / 2) as f64);

    let Point2 { x: x1, y: y1 } = to_camera_space(p1, camera);
    let Point2 { x: x2, y: y2 } = to_camera_space(p2, camera);

    let x_start = x_mid + x1 * camera.scale;
    let y_start = y_mid + y1 * camera.scale;
    let x_end = x_mid + x2 * camera.scale;
    let y_end = y_mid + y2 * camera.scale;

    gl.draw(args.viewport(), |c, gl| {
        let transform = c.transform;
        let l = [x_start, y_start, x_end, y_end];
        line(color, width, l, transform, gl);
    });
}

fn to_camera_space(p: &Point, camera: &Camera) -> Point2<f64> {
    use CameraProjection::*;
    match camera.proj {
        Orthographic => {
            let Point { x, y, z: _ } = *p;
            Point2::new(x, y)
        }
        Perspective => {
            let camera_point = Point::new(0.0, 0.0, 1.0);
            let up = Vector::new(0.0, 1.0, 0.0);
            let look = Matrix4::look_at(camera.pos, camera_point, up);
            let vec = Vector4::new(p.x, p.y, p.z, 1.0);
            let vec_pp = look * vec;
            Point2::new(vec_pp.x, vec_pp.y)
        }

    }
}
