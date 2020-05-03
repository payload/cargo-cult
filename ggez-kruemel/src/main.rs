// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use rand::prelude::*;

use palette::*;

// use std::default;

#[macro_use]
extern crate bitflags;

fn main() {
    experiment_tick_cells();

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

fn experiment_tick_cells() {
    let mut cells = Cells::new(11, 11);
    //cells.paint(5, 0, Sand);
    cells.paint(5, 1, Sand);
    cells.paint(5, 2, Sand);
    print_frames(16, cells);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_cells() { experiment_tick_cells(); }
}

struct Cells {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    tick_n: usize,
}

type X = i32;
type Y = i32;
impl Cells {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::empty(); width * height],
            tick_n: 0,
        }
    }

    fn sim_update(&mut self) {
        self.tick();
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
                    Sand if update_sand => { self.update_sand(x, y, idx); },
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

    fn format(&self) -> String {
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

    fn paint(&mut self, x: X, y: Y, id: CellId) {
        if self.in_bounds(x, y) {
            let idx = self.idx(x, y);
            self.cells[idx] = id.into();
        }
    }

    fn cell_id(&self, x: X, y: Y) -> CellId {
        self.cell(x, y).id
    }

    fn cell(&self, x: X, y: Y) -> Cell {
        if let Some(idx) = self.checked_idx(x, y) {
            if !self.cells[idx].touched() {
                return self.cells[idx];
            } 
        }
        Cell::unavailable()
    }

    fn update_sand(&mut self, x: X, y: Y, idx: usize)  -> (X, Y) {
        let mut cell = self.cells[idx];
        let mut y = y;
        let mut d = self.cell(x, y + 1);
        
        // vy > 0 may be okay, maybe try also d.dy > cell.dy and Empty.dy = 127
        if !(d.id == Empty || d.vy > 0) {
            return (x, y);
        }

        cell.vy += 1;
        cell.dy += cell.vy;

        while cell.dy > 10 && d.id == Empty {
            cell.dy -= 10;
            self.cells[idx] = self.cells[self.idx(x, y + 1)];
            y += 1;
            d = self.cell(x, y + 1);
        }

        if d.id != Empty {
            cell.dy = d.dy;
            cell.vy = d.vy;
        }

        let idx = self.idx(x, y);
        self.cells[idx] = cell;

        (x, y)
    }

    fn update_water(&mut self, x: X, y: Y, idx: usize) -> (X, Y) {
        let d = self.cell_id(x, y + 1);
        let dl = self.cell_id(x - 1, y + 1);
        let dr = self.cell_id(x + 1, y + 1);
        let u = self.cell_id(x, y - 1);
        let l = self.cell_id(x - 1, y);
        let r = self.cell_id(x + 1, y);
        let ul = self.cell_id(x - 1, y - 1);
        let ur = self.cell_id(x + 1, y - 1);
        let rand = |p| random::<f32>() < p;
        let bias: bool = random();

        // fall down
        if d == Empty {
            return self.swap(idx, x, y + 1)
        }

        let left = dl == Empty && l != Sand && l != Water;
        let right = dr == Empty && r != Sand && r != Water;
        if (left && right && bias) || (left && !right) {
            return self.swap(idx, x - 1, y + 1);
        } else if right {
            return self.swap(idx, x + 1, y + 1);
        }

        // spread to side on water surface or under falling sand
        let left = dl == Water && l == Empty;
        let right = dr == Water && r == Empty;
        if (left || right) && u != Water {
            if (left && right && bias) || (left && !right) {
                return self.swap(idx, x - 1, y)
            } else if right {
                return self.swap(idx, x + 1, y)
            }
        }

        // trickle through falling sand
        // (left and right is not Empty)
        if (d == Water || l == Water || r == Water) && (u == Sand || ul == Sand || ur == Sand) {            
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
        let b_idx = self.idx(x, y);
        let mut a = self.cells[a_idx];
        let mut b = self.cells[b_idx];
        let a_touched = a.id != Empty;
        let b_touched = b.id != Empty;
        a.set_touched(a_touched);
        b.set_touched(b_touched);
        self.cells[a_idx] = b;
        self.cells[b_idx] = a;
        (x, y)
    }

    fn swap(&mut self, a_idx: usize, x: X, y: Y) -> (X, Y) {
        let b_idx = self.idx(x, y);
        let a = self.cells[a_idx];
        let b = self.cells[b_idx];
        self.cells[a_idx] = b;
        self.cells[b_idx] = a;
        (x, y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    vx: i8,
    vy: i8,
    dx: i8,
    dy: i8,
    random: f32,
    flags: CellFlags,
    id: CellId,
}

bitflags! {
    struct CellFlags: u8 {
        const TOUCHED = 0b00000001;
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Cell {
    fn new(id: CellId) -> Self { Self { vx: 0, vy: 0, dx: 0, dy: 0, random: random(), flags: CellFlags::empty(), id } }
    fn empty() -> Self { Self { vy: 10, ..Self::new(Empty) } }
    fn sand() -> Self { Self::new(Sand) }
    fn water() -> Self { Self::new(Water) }
    fn wood() -> Self { Self::new(Wood) }
    fn unavailable() -> Self { Self::new(Unavailable) }
    
    fn set_touched(&mut self, set: bool) { self.flags.set(CellFlags::TOUCHED, set) }
    fn touched(&self) -> bool { self.flags.contains(CellFlags::TOUCHED) }

    fn char(&self) -> char {
        match self.id {
            Empty => ' ',
            Sand => '.',
            Water => '~',
            Wood => '#',
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
            Wood => Cell::wood(),
            Unavailable => Cell::unavailable(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellId {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Wood = 3,
    Unavailable = 255,
}

struct MyGame {
    cells: Cells,

    paint_primary_id: CellId,
    paint_secondary_id: CellId,
    paint_size: u32,
    scale: u32,
    
    paused: bool,
}

impl MyGame {
    fn new(ctx: &mut Context) -> MyGame {
        let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
        let scale = 8;
        let w = size.width as u32 / scale;
        let h = size.height as u32 / scale;
        let cells = Cells::new(w as usize, h as usize);

        let game = MyGame {
            cells,
            paint_primary_id: Sand,
            paint_secondary_id: Empty,
            paint_size: 4,
            scale,
            paused: false,
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
                    self.cells.paint(x + dx, y + dy, self.paint_primary_id);
                }
            });
        } else if button_pressed(ctx, MouseButton::Right) {
            for_circle(self.paint_size, &mut |dx, dy| {
                if rand(0.9) {
                    self.cells.paint(x + dx, y + dy, self.paint_secondary_id);
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
        self.draw(ctx)?;
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

            Key1 if !shift => self.paint_primary_id = Empty,
            Key2 if !shift => self.paint_primary_id = Sand,
            Key3 if !shift => self.paint_primary_id = Water,
            Key4 if !shift => self.paint_primary_id = Wood,
            Key1 if shift => self.paint_secondary_id = Empty,
            Key2 if shift => self.paint_secondary_id = Sand,
            Key3 if shift => self.paint_secondary_id = Water,
            Key4 if shift => self.paint_secondary_id = Wood,

            _ => (),
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.paint_size = (self.paint_size as i32 + y.ceil() as i32).max(0) as u32;
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

impl MyGame {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let s = self.scale as i32;
        let mut builder = graphics::MeshBuilder::new();

        let mut draw = |x, y, c: (f32, f32, f32)| {
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
                    Sand => draw(x, y, rgb(hsl(40.0, 1.0, 0.3 + 0.2 * cell.random))),
                    Wood => draw(x, y, rgb(hsl(20.0, 0.6, 0.2 + 0.2 * cell.random))),
                    Water => draw(x, y, rgb(hsl(220.0, 1.0, 0.3 + 0.2 * cell.random))),
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

fn hsl(h: f32, s: f32, l: f32) -> Hsl<> {
    Hsl::new(h, s, l)
}

fn rgb(hsl: Hsl<>) -> (f32, f32, f32) {
    Srgb::from(hsl).into_components()
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

fn print_frames(n: usize, mut cells: Cells) {
    let mut frames: Vec<_> = (0..n).map(|_| {
        let frame: Vec<String> = cells.format().split('\n').map(String::from).collect();
        cells.tick();
        frame.into_iter()
    }).collect();

    let mut end = false;
    while !end {
        let mut line = String::new();
        for frame in frames.iter_mut() {
            if let Some(part) = frame.next() {
                line.push_str(&part);
                line.push_str(" ");
            } else {
                end = true;
            }
        }
        println!("{}", line);
    }
}
