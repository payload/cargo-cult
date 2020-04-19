use gfx::{self, *};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult};

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("game_name", "author_name")
        .build()
        .unwrap();

    let mut my_game = MyGame::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

gfx_defines! {
    constant Dim {
        rage: f32 = "u_Rate",
    }
}

struct MyGame {
    universe: sands::Universe,
    sand_shader: Option<graphics::Shader<Dim>>,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame {
            universe: sands::Universe::new(200, 200),
            sand_shader: None,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.universe.tick();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        if keyboard::is_key_pressed(ctx, keyboard::KeyCode::B) {
            self.draw_black_pixels(ctx)?;
        }
        else if keyboard::is_key_pressed(ctx, keyboard::KeyCode::R) {
            self.draw_raw(ctx)?;
        }
        else if keyboard::is_key_pressed(ctx, keyboard::KeyCode::S) {
            self.draw_with_shader(ctx)?;
        } else {
            self.draw_with_shader(ctx)?;
        }

        graphics::present(ctx)
    }
}

impl MyGame {
    fn draw_with_shader(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(ref shader) = self.sand_shader {
            let _lock = graphics::use_shader(ctx, shader);
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
        let image = graphics::Image::from_rgba8(ctx, width, height, rgba)?;
        graphics::draw(ctx, &image, graphics::DrawParam::default())
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
