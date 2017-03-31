extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Colored, Textured, GlGraphics, OpenGL};
use rand::Rng;
use std::time::SystemTime;
use graphics::math::hsv;
use graphics::types::Color;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

pub struct App {
    time : f64,
    gl: GlGraphics,
    wavefront : Vec<Point>,
    wavefront_prev : Vec<Point>,
    params : VFParams,
}

#[derive(Clone)]
struct Point(f64, f64, f64);

struct VFParams {
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

const PI: f64 = 3.141;

fn plot_points(gl : &mut GlGraphics, args : &RenderArgs, wavefront : &[Point], wavefront_prev : &[Point]){
    //  Assume wavefront length = wavefront_prev length
    for (i, p) in wavefront.iter().enumerate() {
        let color = gen_color(i as f64 / wavefront.len() as f64);
        line_points(gl, args, color, &p, &wavefront_prev[i]);
    }
}

fn gen_color(i : f64) -> Color {
    hsv(RED, (i * 2.0 * PI) as f32, 0.8, 1.0)
}


fn line_points(gl : &mut GlGraphics, args : &RenderArgs, color : Color, p1 : &Point, p2 : &Point) {
    use graphics::*;

    let (x_mid, y_mid) = ((args.width / 2) as f64, (args.height / 2) as f64);
    let scale = 50.0;

    //For now ignore view angle and z
    let Point(x1, y1, _) = *p1;
    let Point(x2, y2, _) = *p2;

    let x_start = x_mid + x1 * scale;
    let y_start = y_mid + y1 * scale;
    let x_end = x_mid + x2 * scale;
    let y_end = y_mid + y2 * scale;

    gl.draw(args.viewport(), |c, gl| {
        let transform = c.transform;
        let l = [x_start, y_start, x_end, y_end];
        line(color, 1.0, l, transform, gl);
    });
}

fn generate_init_points(offset : f64, r : f64, n : usize) -> Vec<Point> {
    let mut points = Vec::new();
    for i in 1..n * 2 {
        let i_f = i as f64;
        let n_f = n as f64;
        let x = r * (offset + (i_f / n_f) *PI).cos();
        let y = r * (offset + (i_f / n_f) *PI).sin();
        let z = r * (offset + (i_f / n_f) *PI).sin();
        let p = Point(x, y, z);
        points.push(p);
    }
    points
}

fn mutate_point(dt: f64, p: &Point, params: &VFParams) -> Point {
    let Point(x, y, z) = *p;
    let Point(i, j, k) = vector_field(p, params);
    Point(x + dt * i, y + dt * j, z + dt * k)
}

fn vector_field(p: &Point, params: &VFParams) -> Point {
    let Point(x, y, z) = *p;

    let shuffle = vec![x, y, z];

    let i = params.q *
            (shuffle[params.a3] /
             (shuffle[params.a2] + params.p * (shuffle[params.a1] + params.q) * PI + z) *
             PI)
        .cos();
    let j = params.r *
            (shuffle[params.a2] * (PI * x).cos() /
             (shuffle[params.a2] + params.r * -(x + params.s)) * PI + x)
        .sin();
    let k = params.w *
            (shuffle[params.a1] /
             (shuffle[params.a2] +
              params.v / (shuffle[params.a3] - shuffle[params.a2] + params.w)) *
             PI + shuffle[params.a1])
        .sin();

    Point(i, j, k)
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            //clear(BLACK, gl);
        });
        plot_points(&mut self.gl, args, &self.wavefront, &self.wavefront_prev);
        self.wavefront_prev = self.wavefront.clone();
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.time += 1.0;
        
        self.wavefront = self.wavefront.iter().map(|p| mutate_point(40.0 * args.dt, p, &self.params)).collect();
    }
}

fn main() {
    let opengl = OpenGL::V4_3;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [800, 600])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let c = Colored::new(opengl.to_glsl());
    let t = Textured::new(opengl.to_glsl());
    let glgraphics = GlGraphics::from_colored_textured(c, t);

    let initial_points = generate_init_points(0.0, 1.0, 50);

    let params = VFParams {
        a1 : 1,
        a2 : 2,
        a3 : 0,
        p : 0.0,
        q : 0.1,
        r : 0.1,
        s : 0.5,
        v : 0.1,
        w : 0.2,
        u : 0.2,
    };

    let mut app = App {
        time : 0.0,
        gl: glgraphics,
        wavefront : initial_points.clone(),
        wavefront_prev : initial_points,
        params : params,
    };

    let mut prev_time = SystemTime::now();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {

            //  Print framerate
            let dt = prev_time.elapsed().unwrap();
            prev_time = SystemTime::now();
            print!("fps {:.3}\r",
                   1000.0 * 1000.0 * 1000.0 / ((dt.subsec_nanos())) as f64);

            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
