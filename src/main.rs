use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics;

fn main() {
    let (mut ctx, mut event_loop) =
    ContextBuilder::new("game_name", "author_name")
        .build()
        .unwrap();
    
    let mut my_game = MyGame::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

struct MyGame {
    universe: sands::Universe
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame {
            universe: sands::Universe::new(200, 200)
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
        self.draw_black_pixels(ctx)?;
        graphics::present(ctx)
    }
}

impl MyGame {
    fn draw_black_pixels(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut builder = graphics::MeshBuilder::new();

        let u = &self.universe;
        let w = u.width();
        u.cells().iter().enumerate().for_each(|(i, c): (usize, &sands::Cell)| {
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
