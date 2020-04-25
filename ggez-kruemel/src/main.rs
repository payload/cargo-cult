// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("game_name", "author_name")
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

struct MyGame {}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let _size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        let game = MyGame {};
        game
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        let _shift = keymods == KeyMods::SHIFT;
        match keycode {
            KeyCode::Escape => ggez::event::quit(ctx),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

impl MyGame {}
