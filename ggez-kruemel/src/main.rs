// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

// use std::default;

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

use CellId::*;

#[test]
fn test_tick_cells() {
    let mut cells = Cells::new(3, 5);
    cells.paint(1, 0, Sand);
    cells.paint(1, 1, Sand);
    cells.paint(1, 2, Sand);
    cells.paint(1, 3, Sand);

    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
}

pub struct Cells {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    tick_n: usize,
}

type X = i32;
type Y = i32;
impl Cells {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, cells: vec![Cell::empty(); width * height], tick_n: 0 }
    }

    pub fn tick(&mut self) {
        let tick_n = self.tick_n;
        let w = self.w();
        let h = self.h();
        let bottom_to_top = (0..h).rev();
        let left_right = |x| if tick_n & 1 == 1 { w - x - 1 } else { x };
        
        for y in bottom_to_top {
            for x in 0..w {
                self.update(left_right(x), y);
            }
        }
        self.tick_n += 1;
    }

    pub fn format(&self) -> String {
        let w = self.width;
        let h = self.height;
        let len = (w + 3) * (h + 2);
        let mut str = String::with_capacity(len);
        str.push_str(&"-".repeat(self.width + 2));
        str.push('\n');
        for y in 0..h {
            str.push('|');
            for x in 0..w {
                str.push(self.cell(x as X, y as Y).char())
            }
            str.push_str("|\n");
        }
        str.push_str(&"-".repeat(w + 2));
        str.push('\n');
        str
    }

    fn w(&self) -> X { self.width as X }
    fn h(&self) -> Y { self.height as Y }
    fn idx(&self, x: X, y: Y) -> usize { (y * self.w() + x) as usize }
    pub fn cell(&self, x: X, y: Y) -> &Cell { &self.cells[self.idx(x, y)] }
    fn in_bounds(&self, x: X, y: Y) -> bool { x >= 0 && x < self.w() && y >= 0 && y < self.h() }
    fn c(&self, idx: usize) -> &Cell { &self.cells[idx] }
    fn c_copy(&self, idx: usize) -> Cell { self.cells[idx] }

    pub fn paint(&mut self, x: X, y: Y, id: CellId) {
        if self.in_bounds(x, y) {
            let idx = self.idx(x, y);
            self.cells[idx] = match id {
                Empty => Cell::empty(),
                Sand => Cell::sand(),
                Water => Cell::water(),
            }
        }
    }

    fn update(&mut self, x: X, y: Y) {
        let idx = self.idx(x, y);
        let id = self.cells[idx].id;
        match id {
            Empty => {},
            Sand => self.update_sand(x, y, idx),
            Water => self.update_water(x, y, idx),
        }
    }

    fn update_sand(&mut self, x: X, y: Y, idx: usize) {
        let id = self.idx(x, y + 1);
        let idl = self.idx(x - 1, y + 1);
        let idr = self.idx(x + 1, y + 1);
        let d_free = y + 1 < self.height as i32;
        let l_free = x > 0;
        let r_free = x + 1 < self.width as i32;
            
        if d_free {
            let d = self.c(id);
            let dl = self.c(idl);
            let dr = self.c(idr);

            if d.touched == 0 && d.id == Empty || d.id == Water {
                self.cells[idx].id = d.id;
                self.cells[id].id = Sand;
            } else if l_free && (dl.id == Empty || dl.id == Water) {
                self.cells[idx].id = dl.id;
                self.cells[idl].id = Sand;
            } else if r_free && (dr.id == Empty || dr.id == Water) {
                self.cells[idx].id = dr.id;
                self.cells[idr].id = Sand;
            }
        }
    }

    fn update_water(&mut self, x: X, y: Y, idx: usize) {
        let d = self.idx(x, y + 1);
        let dl = self.idx(x - 1, y + 1);
        let dr = self.idx(x + 1, y + 1);
        let d_free = y + 1 < self.height as i32;
        let l_free = x > 0;
        let r_free = x + 1 < self.width as i32;
        
        if d_free {
            if self.cells[d].id == Empty {
                self.cells[idx].id = Empty;
                self.cells[d].id = Water;
            } else if l_free && self.cells[dl].id == Empty {
                self.cells[idx].id = Empty;
                self.cells[dl].id = Water;
            } else if r_free && self.cells[dr].id == Empty {
                self.cells[idx].id = Empty;
                self.cells[dr].id = Water;
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub vx: f32,
    pub vy: f32,
    pub time: u8,
    pub touched: u8,
    pub id: CellId,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Cell {
    pub fn empty() -> Self { Self { vx: 0.0, vy: 0.0, time: 0, touched: 0, id: CellId::Empty } }
    pub fn sand() -> Self { Self { id: CellId::Sand, ..Self::empty() } }
    pub fn water() -> Self { Self { id: CellId::Water, ..Self::empty() } }

    pub fn char(&self) -> char {
        match self.id {
            Empty => ' ',
            Sand => '.',
            Water => '~',
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellId {
    Empty = 0,
    Sand = 1,
    Water = 2,
}

struct MyGame {
    cells: Cells,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        let w = size.width as usize / 4;
        let h = size.height as usize / 4;
        let cells = Cells::new(w, h);

        let game = MyGame { cells };
        game
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        use ggez::input::mouse::*;
        let p = position(ctx);
        let x = p.x as i32 / 4;
        let y = p.y as i32 / 4;
        let rand = |p| rand::random::<f64>() < p;

        if button_pressed(ctx, MouseButton::Left) {
            for_rectangle(4, 3, &mut |dx, dy|
                if rand(0.9) {
                    self.cells.paint(x + dx, y + dy, Sand);
                }
            );
        } else if button_pressed(ctx, MouseButton::Right) {
            for_rectangle(4, 3, &mut |dx, dy|
                if rand(0.9) { 
                    self.cells.paint(x + dx, y + dy, Water);
                }
            );
        }

        self.cells.tick();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        self.draw_black_pixels(ctx)?;
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

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

impl MyGame {
    fn draw_black_pixels(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut builder = graphics::MeshBuilder::new();
        
        let mut draw = |x, y, c: (f32, f32, f32, f32)| {
            builder.rectangle(
                graphics::DrawMode::fill(),
                graphics::Rect::new_i32(x * 4, y * 4, 4, 4),
                c.into(),
            );
        };

        for y in 0..self.cells.h() {
            for x in 0..self.cells.w() {
                match self.cells.cell(x, y).id {
                    Empty => {},
                    Sand => draw(x, y, (1.0, 0.8, 0.0, 1.0)),
                    Water => draw(x, y, (0.0, 0.0, 1.0, 1.0)),
                }
            }
        }

        if let Ok(mesh) = builder.build(ctx) {
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())
        } else {
            Ok(())
        }
    }
}

fn for_rectangle<F>(w: i32, h: i32, f: &mut F) where F: FnMut(i32, i32) {
    for dy in 0..h {
        for dx in 0..w {
            f(dx, dy);
        }
    }
}