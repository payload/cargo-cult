// Copyright (c) 2020 Gilbert Röhrbein
use gfx::{self, *};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{is_key_pressed, KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use ggez_sandspiel as sands;

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("game_name", "author_name")
        .add_resource_path("resources")
        .window_setup(ggez::conf::WindowSetup {
            icon: "".into(),
            samples: ggez::conf::NumSamples::Zero,
            srgb: false,
            title: "title".into(),
            vsync: true,
        })
        .build()
        .unwrap();

    let mut my_game = MyGame::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

gfx_defines! {
    constant SandShaderConsts {
        t: f32 = "t",
        is_snapshot: bool = "isSnapshot",
    }
}

type SandShader = graphics::Shader<SandShaderConsts>;

struct MyGame {
    universe: Option<sands::Universe>,
    sand_shader: Option<SandShader>,
    sand_shader_consts: SandShaderConsts,
    paint_size: i32,
    paint_species: sands::Species,
    render_scale: f32,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let mut game = MyGame {
            universe: None,
            sand_shader: None,
            sand_shader_consts: SandShaderConsts {
                t: 0.0,
                is_snapshot: false,
            },
            paint_size: 10,
            paint_species: sands::Species::Water,
            render_scale: 4.0,
        };

        game.universe = game.create_universe(ctx);
        game.reload_resources(ctx);
        game
    }

    fn load_sand_shader(ctx: &mut Context, consts: SandShaderConsts) -> Option<SandShader> {
        match graphics::Shader::new(
            ctx,
            "/sand_shader.glslv",
            "/sand_shader.glslf",
            consts,
            "SandShaderConsts",
            None,
        ) {
            Ok(shader) => Some(shader),
            Err(err) => {
                dbg!(err);
                None
            }
        }
    }

    fn reload_resources(&mut self, ctx: &mut Context) {
        self.sand_shader = Self::load_sand_shader(ctx, self.sand_shader_consts);
        println!("reload_resources done");
    }

    fn create_universe(&self, ctx: &mut Context) -> Option<sands::Universe> {
        let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        Some(sands::Universe::new(
            (size.width as f32 / self.render_scale) as i32,
            (size.height as f32 / self.render_scale) as i32,
        ))
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(ref mut u) = self.universe {
            u.tick();
        }

        self.sand_shader_consts = SandShaderConsts {
            t: ggez::timer::ticks(ctx) as f32,
            ..self.sand_shader_consts
        };

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        if is_key_pressed(ctx, KeyCode::B) {
            self.draw_black_pixels(ctx)?;
        } else if is_key_pressed(ctx, KeyCode::N) {
            self.draw_raw(ctx)?;
        } else if is_key_pressed(ctx, KeyCode::M) {
            self.draw_with_shader(ctx)?;
        } else {
            self.draw_with_shader(ctx)?;
        }

        use graphics::BLACK;
        let text = graphics::Text::new(format!("{:?}", self.paint_species));
        graphics::draw(ctx, &text, ([0.0, 0.0], BLACK))?;

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        use sands::Species::*;
        let shift = keymods == KeyMods::SHIFT;
        match keycode {
            KeyCode::Escape => ggez::event::quit(ctx),
            KeyCode::R if shift => self.universe = self.create_universe(ctx),
            KeyCode::R => self.reload_resources(ctx),

            KeyCode::Add => self.render_scale = self.render_scale + 1.0,
            KeyCode::Subtract => self.render_scale = (self.render_scale - 1.0).max(1.0),

            KeyCode::Key1 if shift => self.paint_species = Plant,
            KeyCode::Key2 if shift => self.paint_species = Acid,
            KeyCode::Key3 if shift => self.paint_species = Stone,
            KeyCode::Key4 if shift => self.paint_species = Dust,
            KeyCode::Key5 if shift => self.paint_species = Mite,
            KeyCode::Key6 if shift => self.paint_species = Oil,
            KeyCode::Key7 if shift => self.paint_species = Rocket,
            KeyCode::Key8 if shift => self.paint_species = Fungus,
            KeyCode::Key9 if shift => self.paint_species = Seed,

            KeyCode::Key0 => self.paint_species = Empty,
            KeyCode::Key1 => self.paint_species = Wall,
            KeyCode::Key2 => self.paint_species = Sand,
            KeyCode::Key3 => self.paint_species = Water,
            KeyCode::Key4 => self.paint_species = Gas,
            KeyCode::Key5 => self.paint_species = Cloner,
            KeyCode::Key6 => self.paint_species = Fire,
            KeyCode::Key7 => self.paint_species = Wood,
            KeyCode::Key8 => self.paint_species = Lava,
            KeyCode::Key9 => self.paint_species = Ice,

            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        if ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left) {
            if let Some(ref mut universe) = self.universe {
                universe.paint(
                    (x / self.render_scale) as i32,
                    (y / self.render_scale) as i32,
                    self.paint_size,
                    self.paint_species,
                );
            }
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        self.universe = self.create_universe(ctx);
    }
}

impl MyGame {
    fn draw_with_shader(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(ref shader) = self.sand_shader {
            let _lock = graphics::use_shader(ctx, shader);
            shader.send(ctx, self.sand_shader_consts)?;
            self.draw_raw(ctx)
        } else {
            self.draw_raw(ctx)
        }
    }

    fn draw_raw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(ref u) = self.universe {
            let width = u.width() as u16;
            let height = u.height() as u16;
            let cells = u.cells();

            let rgba =
                unsafe { std::slice::from_raw_parts(cells.as_ptr() as *const u8, cells.len() * 4) };
            let mut image = graphics::Image::from_rgba8(ctx, width, height, rgba)?;
            image.set_filter(graphics::FilterMode::Linear);

            graphics::draw(
                ctx,
                &image,
                graphics::DrawParam::default().scale([self.render_scale, self.render_scale]),
            )?;
        }
        Ok(())
    }

    fn draw_black_pixels(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut builder = graphics::MeshBuilder::new();

        if let Some(ref u) = self.universe {
            let w = u.width();
            for (i, c) in u.cells().iter().enumerate() {
                if c.species != sands::Species::Empty {
                    let x = (i % w as usize) as i32;
                    let y = (i / w as usize) as i32;
                    builder.rectangle(
                        graphics::DrawMode::fill(),
                        graphics::Rect::new_i32(x, y, 1, 1),
                        (0.0, 0.0, 0.0, 1.0).into(),
                    );
                }
            }
        }

        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())
    }
}
