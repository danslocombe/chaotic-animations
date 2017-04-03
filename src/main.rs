extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use graphics::math::hsv;
use graphics::types::Color;
use opengl_graphics::{Colored, GlGraphics, OpenGL, Textured};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::distributions::{IndependentSample, Range};
use std::path::Path;
use std::time::SystemTime;
use std::f64::consts::PI;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

pub struct Simulation {
    pub params: VFParams,

    time: f64,
    time_max: f64,
    sim_speed: f64,
    wavefront: Vec<Point>,
    wavefront_prev: Vec<Point>,
    point_offset: f64,
    point_radius: f64,
    point_count: usize,
}

pub struct App {
    pub sim: Simulation,

    gl: GlGraphics,
    font: GlyphCache<'static>,
    draw_width: f64,
    sim_speed_mod: f64,
    plot_style: PlotStyle,
}

#[derive(Clone)]
pub struct Point(f64, f64, f64);

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
    fn gen_rand() -> Self {
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

    fn to_string(&self) -> String {
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

enum PlotStyle {
    Point,
    Line,
    Radial,
}

fn plot_points(gl: &mut GlGraphics,
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
                line_points(gl, args, color, width, p, &wavefront_prev[i]);
            }
            PlotStyle::Line => {
                if i > 0 {
                    line_points(gl, args, color, width, p, &wavefront[i - 1]);
                }
            }
            PlotStyle::Radial => {
                line_points(gl, args, color, width, p, &Point(0.0, 0.0, 0.0));
            }
        }
    }
}

fn gen_color(i: f64) -> Color {
    hsv(RED, (i * 2.0 * PI) as f32, 0.8, 1.0)
}


fn line_points(gl: &mut GlGraphics,
               args: &RenderArgs,
               color: Color,
               width: f64,
               p1: &Point,
               p2: &Point) {
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
        line(color, width, l, transform, gl);
    });
}

fn generate_init_points(offset: f64, r: f64, n: usize) -> Vec<Point> {
    let mut points = Vec::new();
    for i in 1..n * 2 {
        let i_f = i as f64;
        let n_f = n as f64;
        let x = r * (offset + (i_f / n_f) * PI).cos();
        let y = r * (offset + (i_f / n_f) * PI).sin();
        let z = r * (offset + (i_f / n_f) * PI).sin();
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

    Point(i, j, k)
}

impl App {
    fn new(glgraphics: GlGraphics,
           params: VFParams,
           plot_style: PlotStyle)
           -> Self {

        let simulation = Simulation {
            time: 0.0,
            time_max: 100.0,
            sim_speed: 0.3,
            wavefront: Vec::new(),
            wavefront_prev: Vec::new(),
            params: params,
            point_offset: 0.0,
            point_radius: 1.0,
            point_count: 100,
        };

        let mut app = App {
            sim: simulation,
            draw_width: 1.0,
            sim_speed_mod: 0.01,
            gl: glgraphics,
            font: GlyphCache::new(Path::new("fonts/alterebro.ttf")).unwrap(),
            plot_style: plot_style,
        };

        app.reset_wavefront();

        app
    }
    fn reset_wavefront(&mut self) {
        let init_points = generate_init_points(self.sim.point_offset,
                                               self.sim.point_radius,
                                               self.sim.point_count);
        self.sim.wavefront = init_points.clone();
        self.sim.wavefront_prev = init_points;
    }
    fn render(&mut self, fps: f64, args: &RenderArgs) {
        use graphics::*;
        self.gl.draw(args.viewport(), |_, gl| {
            // Clear the screen.
            clear(BLACK, gl);
        });
        plot_points(&mut self.gl,
                    args,
                    &self.plot_style,
                    self.draw_width,
                    &self.sim.wavefront,
                    &self.sim.wavefront_prev);
        self.sim.wavefront_prev = self.sim.wavefront.clone();

        //  Draw text and overlay
        let text_fps = format!("fps : {:.2}", fps);
        let text_params = self.sim.params.to_string();
        let text_sim_speed = format!("Sim Speed: {:.3}", self.sim.sim_speed);
        const TEXT_CONTROLS: &str = "Controls -   Space: Restart,   Right: \
                                     Next Params,   Left: Prev Params,  Down: \
                                     Reduce Simulation Speed,   Up: Increase \
                                     Simulation Speed";
        let mut text_properties = Text::new(12);
        text_properties.color = WHITE;
        let font_mut = &mut self.font;
        self.gl.draw(args.viewport(), |c, gl| {
            let transform_text = c.transform.trans(5.0, 12.0);
            text_properties.draw(text_fps.as_str(),
                                 font_mut,
                                 &c.draw_state,
                                 transform_text,
                                 gl);
            text_properties.draw(text_params.as_str(),
                                 font_mut,
                                 &c.draw_state,
                                 transform_text.trans(0.0, 20.0),
                                 gl);
            let transform_text_right =
                transform_text.trans(args.viewport().rect[2] as f64 - 100.0, 0.0);
            text_properties.draw(text_sim_speed.as_str(),
                                 font_mut,
                                 &c.draw_state,
                                 transform_text_right,
                                 gl);
            let transform_text_bottom = c.transform
                .trans(5.0, args.viewport().rect[3] as f64 - 20.0);
            text_properties.draw(TEXT_CONTROLS,
                                 font_mut,
                                 &c.draw_state,
                                 transform_text_bottom,
                                 gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.sim.time += 1.0;
        let dt = self.sim.sim_speed * (self.sim.time / self.sim.time_max).sin();
        self.sim.wavefront = self.sim
            .wavefront
            .iter()
            .map(|p| mutate_point(dt, p, &self.sim.params))
            .collect();
    }

    fn reset_time(&mut self) {
        self.sim.time = 0.0;
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

    let params = VFParams::gen_rand();

    let mut app = App::new(glgraphics, params, PlotStyle::Point);
    let mut prev_time = SystemTime::now();

    let mut params_next_stack: Vec<VFParams> = Vec::new();
    let mut params_prev_stack: Vec<VFParams> = Vec::new();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Input::Render(r) => {

                //  Print framerate
                let dt = prev_time.elapsed().unwrap();
                prev_time = SystemTime::now();
                let fps = 1000.0 * 1000.0 * 1000.0 / (dt.subsec_nanos() as f64);
                app.render(fps, &r);
            }

            Input::Update(u) => {
                app.update(&u);
            }
            Input::Press(i) => {
                match i {
                    Button::Keyboard(Key::Down) => {
                        app.sim.sim_speed -= app.sim_speed_mod;
                    }
                    Button::Keyboard(Key::Up) => {
                        app.sim.sim_speed += app.sim_speed_mod;
                    }
                    Button::Keyboard(Key::Space) => {
                        app.reset_time();
                        app.reset_wavefront();
                    }
                    Button::Keyboard(Key::Left) => {
                        if let Some(prev_params) = params_prev_stack.pop() {
                            params_next_stack.push(app.sim.params);
                            app.sim.params = prev_params;
                            app.reset_time();
                            app.reset_wavefront();
                        }
                    }
                    Button::Keyboard(Key::Right) => {
                        params_prev_stack.push(app.sim.params);
                        if let Some(next_params) = params_next_stack.pop() {
                            app.sim.params = next_params;
                        } else {
                            app.sim.params = VFParams::gen_rand();
                        }
                        app.reset_time();
                        app.reset_wavefront();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
