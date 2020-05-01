// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{self, KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use rand::prelude::*;

// use std::default;

#[macro_use]
extern crate bitflags;

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
    let mut cells = Cells::new(11, 11);
    cells.paint(2, 10, Sand);
    cells.paint(8, 10, Sand);
    cells.paint(5, 10, Water);
    cells.paint(4, 10, Water);
    cells.paint(6, 10, Water);

    println!("{}", cells.format());
    cells.tick();
    println!("{}", cells.format());
}

/*
else if ul == Water && r(0.3) {
            self.swap_touch(idx, x - 1, y + 1);
        } else if ur == Water && r(0.3) {
            self.swap_touch(idx, x + 1, y + 1);
        } 
        */

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
        Self {
            width,
            height,
            cells: vec![Cell::empty(); width * height],
            tick_n: 0,
        }
    }

    fn sim_update(&mut self) {
        for i in 0..1 {
            self.tick();
        }
    }

    fn tick(&mut self) {
        let tick_n = self.tick_n;
        let w = self.w();
        let h = self.h();
        let bottom_to_top = (0..h).rev();
        let left_right = |x| if tick_n & 1 == 1 { w - x - 1 } else { x };
        let update_sand = true;

        for y in bottom_to_top {
            for x in 0..w {
                let x = left_right(x);
                let idx = self.idx(x, y);
                let cell = self.cells[idx];
                if cell.touched() { continue }
                match cell.id {
                    Sand if update_sand => self.update_sand(x, y, idx),
                    Water => { self.update_water(x, y, idx); },
                    _ => {},
                }
            }
        }

        let mut count = [0; 256];
        for cell in self.cells.iter_mut() {
            count[cell.id as usize] += 1;
            cell.set_touched(false);
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

    fn w(&self) -> X {
        self.width as X
    }
    fn h(&self) -> Y {
        self.height as Y
    }
    fn idx(&self, x: X, y: Y) -> usize {
        (y * self.w() + x) as usize
    }
    pub fn cell(&self, x: X, y: Y) -> &Cell {
        &self.cells[self.idx(x, y)]
    }
    pub fn mut_cell(&mut self, x: X, y: Y) -> &mut Cell {
        let idx = self.idx(x, y);
        &mut self.cells[idx]
    }
    fn in_bounds(&self, x: X, y: Y) -> bool {
        x >= 0 && x < self.w() && y >= 0 && y < self.h()
    }

    fn checked_idx(&self, x: X, y: Y) -> Option<usize> {
        let idx = self.idx(x, y);
        if self.in_bounds(x, y) {
            Some(idx)
        } else {
            None
        }
    }

    pub fn paint(&mut self, x: X, y: Y, id: CellId) {
        if self.in_bounds(x, y) {
            let idx = self.idx(x, y);
            self.cells[idx] = id.into();
        }
    }

    fn cell_id(&self, x: X, y: Y) -> CellId {
        if let Some(idx) = self.checked_idx(x, y) {
            if !self.cells[idx].touched() {
                return self.cells[idx].id;
            } 
        }
        Unavailable
    }

    fn update_sand(&mut self, x: X, y: Y, idx: usize) {
        let d = self.cell_id(x, y + 1);
        let dl = self.cell_id(x - 1, y + 1);
        let dr = self.cell_id(x + 1, y + 1);

        if d == Empty {
            self.swap_touch(idx, x, y + 1);
        } else if dl == Empty {
            self.swap_touch(idx, x - 1, y + 1);
        } else if dr == Empty {
            self.swap_touch(idx, x + 1, y + 1);
        }
    }

    fn update_water(&mut self, x: X, y: Y, idx: usize) -> (X, Y) {
        let cell = self.mut_cell(x, y);
        let spread = cell.flags.contains(CellFlags::SPREAD);
        cell.flags.remove(CellFlags::SPREAD);
        cell.flags.remove(CellFlags::HIDDEN);

        let d = self.cell_id(x, y + 1);
        let dl = self.cell_id(x - 1, y + 1);
        let dr = self.cell_id(x + 1, y + 1);
        let u = self.cell_id(x, y - 1);
        let ul = self.cell_id(x - 1, y - 1);
        let ur = self.cell_id(x + 1, y - 1);
        let l = self.cell_id(x - 1, y - 1);
        let r = self.cell_id(x + 1, y - 1);
        let rand = |p| random::<f32>() < p;

        // fall down
        if d == Empty {
            return self.swap(idx, x, y + 1)
        } else if dl == Empty {
            return self.swap(idx, x - 1, y + 1)
        } else if dr == Empty {
            return self.swap(idx, x + 1, y + 1)
        }

        // spread to side on water surface or under falling sand
        let left = dl == Water && l == Empty;
        let right = dr == Water && r == Empty;
        if d == Water && (left || right) && (u == Empty || u == Sand) {
            let bias = random::<bool>();
            if (left && right && bias) || (left && !right) {
                return self.swap(idx, x - 1, y)
            } else if right {
                return self.swap(idx, x + 1, y)
            }
        }

        // trickle through falling sand
        // (left and right is not Empty)
        if (d == Water || l == Water || r == Water) && (u == Sand || ul == Sand || ur == Sand) {
            let bias = random::<bool>();
            
            // NOTE: another variant of sand falling, duplicates parts of update_sand
            // TODO: does not return new position

            if u == Empty {
                self.swap_touch(idx, x, y - 1);

                if (bias && ul == Sand && ur == Sand) || (ul == Sand && ur != Sand) {
                    self.swap_touch(idx, x - 1, y - 1);
                } else if ur == Sand {
                    self.swap_touch(idx, x + 1, y - 1);
                }
            } else if u == Water && rand(0.3) {
                if (bias && ul == Sand && ur == Sand) || (ul == Sand && ur != Sand) {
                    self.swap_touch(idx, x - 1, y - 1);
                } else if ur == Sand {
                    self.swap_touch(idx, x + 1, y - 1);
                }
            } else if u == Sand && rand(0.3) {
                if (bias && ul == Empty && ur == Empty) || (ul == Empty && ur != Empty) {
                    self.swap_touch(idx, x - 1, y - 1);
                } else if ur == Empty {
                    self.swap_touch(idx, x + 1, y - 1);
                }

                self.swap_touch(idx, x, y - 1);
            }
        }
            
        (x, y)
    }

    fn swap_touch(&mut self, a_idx: usize, x: X, y: Y) -> (X, Y) {
        let a_id = self.cells[a_idx].id;
        let b_idx = self.idx(x, y);
        let b_id = self.cells[b_idx].id;
        self.cells[a_idx].id = b_id;
        self.cells[a_idx].set_touched(b_id != Empty);
        self.cells[b_idx].id = a_id;
        self.cells[b_idx].set_touched(a_id != Empty);
        (x, y)
    }

    fn swap(&mut self, a_idx: usize, x: X, y: Y) -> (X, Y) {
        let a_id = self.cells[a_idx].id;
        let b_idx = self.idx(x, y);
        let b_id = self.cells[b_idx].id;
        self.cells[a_idx].id = b_id;
        self.cells[b_idx].id = a_id;
        (x, y)
    }

    fn is_empty(&self, x: X, y: Y) -> bool {
        self.cell_id(x, y) == Empty
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    flags: CellFlags,
    pub id: CellId,
}

bitflags! {
    struct CellFlags: u8 {
        const EMPTY   = 0b00000000;
        const TOUCHED = 0b00000001;
        const SURFACE = 0b00000010;
        const HIDDEN  = 0b00000100;
        const SPREAD  = 0b00001000;
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Cell {
    fn new(id: CellId) -> Self { Self { flags: CellFlags::EMPTY, id } }
    pub fn empty() -> Self { Self::new(Empty) }
    pub fn sand() -> Self { Self::new(Sand) }
    pub fn water() -> Self { Self::new(Water) }
    pub fn unavailable() -> Self { Self::new(Unavailable) }

    fn set_flag(&mut self, flag: CellFlags, set: bool) {
        if set {
            self.flags |= flag;
        } else {
            self.flags -= flag;
        }
    }
    fn flag(&self, flag: CellFlags) -> bool {
        self.flags & flag == flag
    }

    pub fn set_hidden(&mut self, set: bool) { self.set_flag(CellFlags::HIDDEN, set) }
    pub fn hidden(&self) -> bool { self.flag(CellFlags::HIDDEN) }

    pub fn set_touched(&mut self, set: bool) { self.set_flag(CellFlags::TOUCHED, set) }
    pub fn touched(&self) -> bool { self.flag(CellFlags::TOUCHED) }

    pub fn set_surface(&mut self, set: bool) { self.set_flag(CellFlags::SURFACE, set) }
    pub fn surface(&self) -> bool { self.flag(CellFlags::SURFACE) }

    pub fn char(&self) -> char {
        match self.id {
            Empty => ' ',
            Sand => '.',
            Water => '~',
            Unavailable => 'X',
        }
    }
}

impl From<CellId> for Cell {
    fn from(id: CellId) -> Cell {
        match id {
            Empty => Cell::empty(),
            Sand => Cell::sand(),
            Water => Cell::water(),
            Unavailable => Cell::unavailable(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellId {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Unavailable = 255,
}

struct MyGame {
    cells: Cells,

    paint_size: u32,
    scale: u32,
    
    paused: bool,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        let scale = 8;
        let w = size.width as u32 / scale;
        let h = size.height as u32 / scale;
        let cells = Cells::new(w as usize, h as usize);

        let game = MyGame {
            cells,
            paint_size: 4,
            scale,
            paused: true,
        };
        game
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        use ggez::input::mouse::*;
        let p = position(ctx);
        let x = p.x as i32 / self.scale as i32;
        let y = p.y as i32 / self.scale as i32;
        let rand = |p| random::<f64>() < p;

        if button_pressed(ctx, MouseButton::Left) {
            for_circle(self.paint_size, &mut |dx, dy| {
                if rand(0.9) {
                    self.cells.paint(x + dx, y + dy, Sand);
                }
            });
        } else if button_pressed(ctx, MouseButton::Right) {
            for_circle(self.paint_size, &mut |dx, dy| {
                if rand(0.9) {
                    self.cells.paint(x + dx, y + dy, Water);
                }
            });
        }

        if !self.paused {
            self.cells.sim_update()
        }

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
        use KeyCode::*;
        let shift = keymods == KeyMods::SHIFT;
        match keycode {
            Escape => ggez::event::quit(ctx),
            U => self.cells.sim_update(),
            P => self.paused = !self.paused,
            Minus => self.scale = 1.max(self.scale - 1),
            Equals if shift => self.scale = self.scale + 1,
            Add => self.scale = self.scale + 1,
            Key0 => self.scale = 8,
            _ => (),
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.paint_size = (self.paint_size as i32 + y.ceil() as i32).max(0) as u32;
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

impl MyGame {
    fn draw_black_pixels(&mut self, ctx: &mut Context) -> GameResult<()> {
        let s = self.scale as i32;
        let mut builder = graphics::MeshBuilder::new();

        let mut draw = |x, y, c: (f32, f32, f32, f32)| {
            builder.rectangle(
                graphics::DrawMode::fill(),
                graphics::Rect::new_i32(x * s, y * s, s, s),
                c.into(),
            );
        };

        for y in 0..self.cells.h() {
            for x in 0..self.cells.w() {
                let cell = self.cells.cell(x, y);
                match cell.id {
                    Sand => draw(x, y, (1.0, 0.8, 0.0, 1.0)),
                    Water => if keyboard::is_key_pressed(ctx, KeyCode::H) && cell.hidden() {
                        draw(x, y, (0.5, 0.0, 1.0, 1.0));
                    } else if !cell.hidden() {
                        draw(x, y, (0.0, 0.0, 1.0, 1.0));
                    },
                    _ => {},
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

fn for_circle<F>(d: u32, f: &mut F)
where
    F: FnMut(i32, i32),
{
    let d = d as i32;
    for dy in -d..=d {
        for dx in -d..=d {
            if dx * dx + dy * dy <= (d * d) as i32 {
                f(dx, dy);
            }
        }
    }
}
