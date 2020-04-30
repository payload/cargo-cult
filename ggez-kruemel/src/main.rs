// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

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
        Self {
            width,
            height,
            cells: vec![Cell::empty(); width * height],
            tick_n: 0,
        }
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

    fn update(&mut self, x: X, y: Y) {
        let idx = self.idx(x, y);
        let id = self.cells[idx].id;
        match id {
            Sand => self.update_sand(x, y, idx),
            Water => self.update_water(x, y, idx),
            _ => {},
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
        let r = |p| rand::random::<f32>() < p;

        if d == Empty {
            self.swap(idx, x, y + 1);
        } else if d == Water && r(0.3) {
            self.swap(idx, x, y + 1);
        } else if dl == Empty {
            self.swap(idx, x - 1, y + 1);
        } else if dr == Empty {
            self.swap(idx, x + 1, y + 1);
        } else if dl == Water && r(0.3) {
            self.swap(idx, x - 1, y + 1);
        } else if dr == Water && r(0.3) {
            self.swap(idx, x + 1, y + 1);
        }
    }

    fn update_water(&mut self, x: X, y: Y, idx: usize) {
        let d = self.cell_id(x, y + 1);
        let dl = self.cell_id(x - 1, y + 1);
        let dr = self.cell_id(x + 1, y + 1);

        if d == Empty {
            self.swap(idx, x, y + 1);
        } else if dl == Empty {
            self.swap(idx, x - 1, y + 1);
        } else if dr == Empty {
            self.swap(idx, x + 1, y + 1);
        } else {
            self.try_spread(x, y);
        }
    }

    fn try_spread(&mut self, x: X, y: Y) {
        if let Some((left, right)) = self.try_spread_get_offsets(x, y) {
            let off = rand_abs_max(left, right);
            self.swap(self.idx(x, y), x + off, y);
        }
    }

    fn try_spread_get_offsets(&self, x: X, y: Y) -> Option<(i32, i32)> {
        let mut left_off = 0;
        let mut right_off = 0;
        for off in 1..5 {
            let left = self.is_empty(x - off, y);
            let right = self.is_empty(x + off, y);
            if !left && !right {
                break;
            }
            if left {
                left_off = -off;
            }
            if right {
                right_off = off;
            }
        }
        if left_off != 0 || right_off != 0 {
            Some((left_off, right_off))
        } else {
            None
        }
    }

    fn swap(&mut self, a_idx: usize, x: X, y: Y) {
        let a_id = self.cells[a_idx].id;
        let b_idx = self.idx(x, y);
        let b_id = self.cells[b_idx].id;
        self.cells[a_idx].id = b_id;
        self.cells[a_idx].set_touched(b_id != Empty);
        self.cells[b_idx].id = a_id;
        self.cells[b_idx].set_touched(a_id != Empty);
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

    pub fn set_touched(&mut self, set: bool) { self.set_flag(CellFlags::TOUCHED, set) }
    pub fn touched(&self) -> bool { self.flags == CellFlags::TOUCHED }

    pub fn set_surface(&mut self, set: bool) { self.set_flag(CellFlags::SURFACE, set) }
    pub fn surface(&self) -> bool { self.flags == CellFlags::SURFACE }

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
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        let w = size.width as usize / 4;
        let h = size.height as usize / 4;
        let cells = Cells::new(w, h);

        let game = MyGame {
            cells,
            paint_size: 4,
        };
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

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.paint_size = (self.paint_size as i32 + y.ceil() as i32).max(1) as u32;
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
                    Sand => draw(x, y, (1.0, 0.8, 0.0, 1.0)),
                    Water => draw(x, y, (0.0, 0.0, 1.0, 1.0)),
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
    let w = d as i32;
    let h = d as i32;
    for dy in -h / 2..h / 2 {
        for dx in -w / 2..w / 2 {
            if dx * dx + dy * dy <= (d * d) as i32 {
                f(dx, dy);
            }
        }
    }
}

fn rand_abs_max(a: i32, b: i32) -> i32 {
    if (a.abs() == b.abs() && rand::random::<bool>()) || a.abs() > b.abs() {
        a
    } else {
        b
    }
}
