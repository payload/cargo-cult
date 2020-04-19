use gfx::{self, *};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{is_key_pressed, KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

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
        dpi: f32 = "dpi",
        resolution: [f32; 2] = "resolution",
        is_snapshot: bool = "isSnapshot",
    }
}

type SandShader = graphics::Shader<SandShaderConsts>;

struct MyGame {
    universe: sands::Universe,
    sand_shader: Option<SandShader>,
    sand_shader_consts: SandShaderConsts,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let mut game = MyGame {
            universe: Self::default_universe(),
            sand_shader: None,
            sand_shader_consts: SandShaderConsts {
                t: 0.0,
                dpi: 1.0,
                resolution: [1.0, 1.0],
                is_snapshot: false,
            },
        };
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

    fn reset_universe(&mut self) {
        self.universe = Self::default_universe();
    }

    fn default_universe() -> sands::Universe {
        let mut universe = sands::Universe::new(100, 100);
        universe.paint(5, 5, 10, sands::Species::Wall);
        universe
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.universe.tick();

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

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => ggez::event::quit(ctx),
            KeyCode::R if keymods == KeyMods::NONE => self.reload_resources(ctx),
            KeyCode::R if keymods == KeyMods::SHIFT => self.reset_universe(),
            KeyCode::I => println!("{:?}", self.universe.cells().last().unwrap().species),
            _ => (),
        }
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
        let u = &self.universe;
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
            graphics::DrawParam::default().scale([4.0, 4.0]),
        )
    }

    fn draw_black_pixels(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut builder = graphics::MeshBuilder::new();

        let u = &self.universe;
        let w = u.width();
        u.cells()
            .iter()
            .enumerate()
            .for_each(|(i, c): (usize, &sands::Cell)| {
                if c.species != sands::Species::Empty {
                    let x = (i % w as usize) as i32;
                    let y = (i / w as usize) as i32;
                    builder.rectangle(
                        graphics::DrawMode::fill(),
                        graphics::Rect::new_i32(x, y, 1, 1),
                        (0.0, 0.0, 0.0, 1.0).into(),
                    );
                }
            });

        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())
    }
}
