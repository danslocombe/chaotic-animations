extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate cgmath;

mod math;
mod render;


use glutin_window::GlutinWindow as Window;
use math::*;
use opengl_graphics::{Colored, GlGraphics, OpenGL, Textured};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use render::*;
use std::f64::consts::PI;
use std::path::Path;
use std::time::SystemTime;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

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
    pub plot_style: PlotStyle,
    pub camera: Camera,

    gl: GlGraphics,
    font: GlyphCache<'static>,
    draw_width: f64,
    sim_speed_mod: f64,
}

fn generate_init_points(offset: f64, r: f64, n: usize) -> Vec<Point> {
    let mut points = Vec::new();
    for i in 1..n * 2 {
        let i_f = i as f64;
        let n_f = n as f64;
        let x = r * (offset + (i_f / n_f) * PI).cos();
        let y = r * (offset + (i_f / n_f) * PI).sin();
        let z = 2.0 * i_f / n_f * r - r;
        let p = Point::new(x, y, z);
        points.push(p);
    }
    points
}

const DEFAULT_SIM_SPEED: f64 = 15.0;

impl App {
    fn new(glgraphics: GlGraphics,
           params: VFParams,
           plot_style: PlotStyle)
           -> Self {

        let simulation = Simulation {
            time: 0.0,
            time_max: 100.0,
            sim_speed: DEFAULT_SIM_SPEED,
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
            sim_speed_mod: 0.1,
            gl: glgraphics,
            font: GlyphCache::new(Path::new("fonts/alterebro.ttf")).unwrap(),
            plot_style: plot_style,
            camera: Camera {
                pos: Point::new(0.0, 0.0, -10.0),
                dir: Vector::new(0.0, 0.0, 1.0),
                scale: 50.0,
                proj: CameraProjection::Orthographic,
            },
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
                    &self.camera,
                    args,
                    &self.plot_style,
                    self.draw_width,
                    &self.sim.wavefront,
                    &self.sim.wavefront_prev);
        self.sim.wavefront_prev = self.sim.wavefront.clone();

        //  Draw text and overlay
        let text_fps = format!("fps : {:.2}", fps);
        let text_params = self.sim.params.to_string();
        let text_camera = match self.camera.proj {
            CameraProjection::Orthographic => "Tab: Perspective View",
            CameraProjection::Perspective => "Tab: Orthographic View",
        };
        let text_sim_speed = format!("Sim Speed: {:.3}", self.sim.sim_speed);
        const TEXT_CONTROLS: &str = "Ctrl : Reset,  Space: Pause Time,   \
                                     Right: Next Params,   Left: Prev Params,  \
                                     Down: Reduce Simulation Speed,   Up: \
                                     Increase Simulation Speed";
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
            text_properties.draw(text_camera,
                                 font_mut,
                                 &c.draw_state,
                                 transform_text.trans(0.0, 40.0),
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

        //  Advance time in sinusoidal pattern based on time_max and sim_speed
        let dt = self.sim.sim_speed * args.dt *
                 (self.sim.time / self.sim.time_max).sin();

        self.camera.update(self.sim.time);

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

    let mut window: Window = WindowSettings::new("demo", [800, 600])
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

    let mut sim_speed_buffer = DEFAULT_SIM_SPEED;

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Input::Render(r) => {
                //  Calculate framerate
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
                    Button::Keyboard(Key::P) => {
                        app.plot_style = PlotStyle::Point;
                    }
                    Button::Keyboard(Key::L) => {
                        app.plot_style = PlotStyle::Line;
                    }
                    Button::Keyboard(Key::R) => {
                        app.plot_style = PlotStyle::Radial;
                    }
                    Button::Keyboard(Key::Down) => {
                        app.sim.sim_speed -= app.sim_speed_mod;
                    }
                    Button::Keyboard(Key::Up) => {
                        app.sim.sim_speed += app.sim_speed_mod;
                    }
                    Button::Keyboard(Key::LCtrl) => {
                        app.reset_time();
                        app.reset_wavefront();
                    }
                    Button::Keyboard(Key::Space) => {
                        if app.sim.sim_speed.abs() > 0.001 {
                            sim_speed_buffer = app.sim.sim_speed;
                            app.sim.sim_speed = 0.0;
                        } else {
                            app.sim.sim_speed = if sim_speed_buffer.abs() >
                                                   0.001 {
                                sim_speed_buffer
                            } else {
                                DEFAULT_SIM_SPEED
                            };
                        }
                    }
                    Button::Keyboard(Key::Tab) => {
                        use CameraProjection::*;
                        app.camera.proj = match app.camera.proj {
                            Orthographic => Perspective,
                            Perspective => Orthographic,
                        }
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
