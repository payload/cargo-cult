// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use rand::prelude::*;
use palette::*;
use line_drawing::*;

// use std::default;

#[macro_use]
extern crate bitflags;

fn main() {
    //experiment_tick_cells();

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
    
    cells.paint(5, 10, Sand);
    cells.paint(5, 9, Sand);
    cells.paint(5, 8, Sand);
    cells.paint(4, 10, Sand);
    cells.paint(4, 9, Sand);
    cells.paint(4, 8, Sand);
    cells.paint(3, 10, Sand);
    cells.paint(3, 9, Sand);
    cells.paint(3, 8, Sand);

    //let idx = cells.idx(5, 5);
    //cells.cells[idx] = Cell { dx: -9, vx: -10, ..Cell::sand() };

    //cells.paint(5, 5, Wood);

    print_frames(7, &mut cells);
    print_frames(2, &mut cells);
    print_frames(2, &mut cells);
    //print_frames(8, &mut cells);
    //print_frames(8, &mut cells);
    //print_frames(8, &mut cells);
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
    loop_count: usize,
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
            loop_count: 0,
        }
    }

    fn sim_update(&mut self) {
        self.loop_count = 0;
        self.tick();
        println!("loop_count {}", self.loop_count);
    }

    fn tick(&mut self) {
        let tick_n = self.tick_n;
        let w = self.w();
        let h = self.h();
        let bottom_to_top = (0..h).rev();
        let left_right = |x| if tick_n & 1 == 1 { w - x - 1 } else { x };
        let update_sand = true;

        for cell in self.cells.iter_mut() {
            cell.flags.remove(CellFlags::TOUCHED);
            cell.flags.remove(CellFlags::TRIED);
            cell.flags.remove(CellFlags::V_FREE);
        }

        for y in bottom_to_top {
            for x in 0..w {
                let x = left_right(x);
                let idx = self.idx(x, y);
                let cell = self.cells[idx];
                if cell.touched() { continue }
                match cell.id {
                    Sand if update_sand => { self.update_sand(x, y, idx); },
                    Water => { self.update_water(x, y, idx); },
                    Wood => { self.update_wood(x, y, idx); }
                    _ => {},
                }
            }
        }

        self.tick_n += 1;
    }

    fn format(&self) -> String {
        let w = self.width;
        let h = self.height;
        let len = (w + 3) * (h + 2);
        let mut str = String::with_capacity(len);
        str.push_str(&"-".repeat(self.width + 3));
        str.push('\n');
        for y in 0..h {
            str.push( y.to_string().chars().last().unwrap());
            str.push('|');
            for x in 0..w {
                str.push(self.cell(x as X, y as Y).char())
            }
            str.push_str("|\n");
        }
        str.push_str(&"-".repeat(w + 3));
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
        if let Some(idx) = self.checked_idx(x, y) {
            if self.cells[idx].id != id {
                self.cells[idx] = id.into();
            }
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

    #[inline(always)]
    fn update_sand_acceleration(&mut self, x: X, y: Y, mut cell: Cell) -> Cell {
        let vx = cell.vx();
        let vy = cell.vy();
        let (h, v) = next_pixel(vx, vy + 1);
        let d = self.cell(x + h, y + v);
        
        if let Some(poo) = self.checked_idx(x + h, y + v) {
            self.cells[poo].flags.insert(CellFlags::V_FREE);
        }

        let accel = 1;
        if d.id == Empty {
            cell.vy = cell.vy.saturating_add(accel);
        } else if d.id == Sand {
            if d.vy > cell.vy {
                cell.vy += (d.vy - cell.vy).min(accel);
            } else
            // If sand stands still v=0, try to ripple down artificially.
            // Alternative idea:
            //  If sand stands perfectly still v=0 d=0, don't ripple down.
            //  But when sand moves, apply at least d=1 to surrounding sand.
            if vx == 0 && vy == 0 {
                let empty_l = self.cell(x - 1, y + 1).id == Empty;
                let empty_r = self.cell(x + 1, y + 1).id == Empty;
                if (empty_l && empty_r && random()) || (empty_l && !empty_r) {
                    cell.vx -= accel;
                    cell.vy += accel;
                    if cell.dy == 0 { // random advantage
                        cell.dy = (cell.random * 5.0).floor() as i8;
                    }
                } else if empty_r {
                    cell.vx += accel;
                    cell.vy += accel;
                    if cell.dy == 0 { // random advantage
                        cell.dy = (cell.random * 5.0).floor() as i8;
                    }
                }
            }
        } else if d.id == Unavailable {
            
        }

        cell
    }

    fn update_sand(&mut self, x: X, y: Y, idx: usize) {
        let mut cell: Cell = self.cells[idx];
        assert_eq!(cell.id, Sand);
        
        cell = self.update_sand_acceleration(x, y, cell);
        
        cell.dx = cell.dx.saturating_add(cell.vx);
        cell.dy = cell.dy.saturating_add(cell.vy);

        if !(cell.dx >= 10 || cell.dy >= 10 || cell.dx <= -10 || cell.dy <= -10) {
            self.cells[idx] = cell;
            return;
        }

        let mut dx = cell.dx();
        let mut dy = cell.dy();
        let mut x0 = x;
        let mut y0 = y;
        let mut x1 = x;
        let mut y1 = y;
        let mut idx0;

        let mut loop_count = 0;
        loop {
            loop_count += 1;
            idx0 = self.idx(x0, y0);
            let ddx = dx / 10;
            let ddy = dy / 10;

            if ddx == 0 && ddy == 0 {
                cell.set_dx(dx);
                cell.set_dy(dy);
                self.cells[idx0] = cell;
                break;
            }

            let (h, v) = next_pixel(ddx, ddy);
            assert!(h != 0 || v != 0);
            x1 = x0 + h;
            y1 = y0 + v;
            
            let idx1 = self.idx(x1, y1);
            let next = self.cell(x1, y1);
            
            if next.id == Empty {
                self.cells[idx0] = self.cells[idx1];
                x0 = x1;
                y0 = y1;
                
                if h != 0 {
                    dx -= 10 * dx.signum();
                }
                if v != 0 {
                    dy -= 10 * dy.signum();
                }

            } else if next.id == Sand {
                self.cells[idx1].flags.insert(CellFlags::TRIED);
                if v != 0 && h == 0 {
                    // check diagonals for empty cells
                    let dr_empty = self.cell(x1 + 1, y1).id == Empty;
                    let dl_empty = self.cell(x1 - 1, y1).id == Empty;
                    if dl_empty && dr_empty {
                        // TODO try alternative, points until 10 instead of /2
                        dx += random_signum(dx) * (dy / 2).abs();
                        dy /= 2;
                    } else if dl_empty && !dr_empty {
                        dx = -dx.abs() - (dy / 2).abs();
                        dy /= 2;
                    } else if dr_empty && !dl_empty {
                        dx = dx.abs() + (dy / 2).abs();
                        dy /= 2;
                    } else {
                        dx = 0;
                        dy = 0;
                    }
                } else {
                    dx = 0;
                    dy = 0;
                }
            } else {
                if h != 0 {
                    dx %= 10;
                    cell.vx = 0;
                }
                if v != 0 {
                    dy %= 10;
                    cell.vy = 0;
                }
            }
        }
        self.loop_count = self.loop_count.max(loop_count);
    }

    fn update_wood(&mut self, x: X, y: Y, _idx: usize) {
        // if let Some(idx) = self.checked_idx(x, y - 1) {
        //     let mut u = self.cell(x, y - 1);
        //     if u.id == Sand {
        //         u.vx = u.vx.saturating_add(-10);
        //         self.cells[idx] = u;
        //     }
        // }
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
        const TRIED   = 0b00000010;
        const V_FREE  = 0b00000100;
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Cell {
    fn new(id: CellId) -> Self { Self { vx: 0, vy: 0, dx: 0, dy: 0, random: random(), flags: CellFlags::empty(), id } }
    fn empty() -> Self { Self { ..Self::new(Empty) } }
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

    fn vx(&self) -> i32 { self.vx as i32 }
    fn vy(&self) -> i32 { self.vy as i32 }
    fn dx(&self) -> i32 { self.dx as i32 }
    fn dy(&self) -> i32 { self.dy as i32 }
    fn set_vx(&mut self, v: i32) { self.vx = v as i8; }
    fn set_vy(&mut self, v: i32) { self.vy = v as i8; }
    fn set_dx(&mut self, v: i32) { self.dx = v as i8; }
    fn set_dy(&mut self, v: i32) { self.dy = v as i8; }
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
        let w = size.width as i32 / scale;
        let h = size.height as i32 / scale;
        // let w = 10;
        // let h = 20;
        let cells = Cells::new(w as usize, h as usize);

        let mut game = MyGame {
            cells,
            paint_primary_id: Sand,
            paint_secondary_id: Empty,
            paint_size: 4,
            scale: scale as u32,
            paused: true,
        };

        let w2 = w/2 - 1;
        // game.cells.paint(w2+0, h-1, Sand);
        game.cells.paint(w2+1, h-1, Sand);
        // game.cells.paint(w2+2, h-1, Sand);
        // game.cells.paint(w2+0, h-2, Sand);
        game.cells.paint(w2+1, h-2, Sand);
        // game.cells.paint(w2+2, h-2, Sand);

        // game.cells.paint(w2-1, h-2, Wood);
        // game.cells.paint(w2+3, h-2, Wood);
        game.cells.paint(w2-0, h-2, Wood);
        game.cells.paint(w2+2, h-2, Wood);

        // game.cells.paint(0, h-1, Sand);
        // game.cells.paint(0, h-2, Sand);
        // game.cells.paint(0, h-3, Sand);
        // game.cells.paint(0, h-4, Sand);
        // game.cells.paint(0, h-5, Sand);
        // game.cells.paint(0, h-6, Sand);

        // game.cells.paint(w2-1, h-1, Wood);
        // game.cells.paint(w2-0, h-1, Wood);
        // game.cells.paint(w2+1, h-1, Wood);
        // game.cells.paint(w2-0, h-2, Sand);

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
                if rand(0.8) {
                    self.cells.paint(x + dx, y + dy, self.paint_primary_id);
                }
            });
        } else if button_pressed(ctx, MouseButton::Right) {
            for_circle(self.paint_size, &mut |dx, dy| {
                if rand(0.8) {
                    self.cells.paint(x + dx, y + dy, self.paint_secondary_id);
                }
            });
        }

        if !self.paused {
            self.cells.sim_update();
            if self.cells.w() < 20 {
                self.paused = true;
            }
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

        for y in 0..=self.cells.h() {
            for x in 0..=self.cells.w() {
                let cell = self.cells.cell(x, y);
                if cell.id != Empty {
                    draw(x, y, cell_color_real(cell));
                }
            }
        }

        if self.cells.w() < 20 {
            for y in 0..=self.cells.h() {
                for x in 0..=self.cells.w() {
                    self.cell_debug(ctx, x as f32, y as f32, &self.cells.cell(x, y));
                    
                }
            }
        }

        if let Ok(mesh) = builder.build(ctx) {
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())
        } else {
            Ok(())
        }
    }

    fn cell_debug(&mut self, ctx: &mut Context, x: f32, y: f32, cell: &Cell) {
        let s = self.scale as f32;
        let w = self.cells.w() as f32;
        let params = graphics::DrawParam::new()
            .color(graphics::BLACK)
            .dest([(x + w) * s * 3.0, y * s * 3.0])
            .scale([0.75, 0.75]);

        if cell.id != Empty {
            graphics::draw(ctx, &graphics::Text::new(cell_text_debug(cell)), params).unwrap();
        } else if cell.flags.contains(CellFlags::V_FREE) {
            graphics::draw(ctx, &graphics::Text::new("x"), params).unwrap();
        }
    }
}

fn cell_color_real(cell: Cell) -> (f32, f32, f32) {
    match cell.id {
        Sand => rgb(hsl(40.0, 1.0, 0.3 + 0.2 * cell.random)),
        Wood => rgb(hsl(20.0, 0.6, 0.2 + 0.2 * cell.random)),
        Water => rgb(hsl(220.0, 1.0, 0.3 + 0.2 * cell.random)),
        _ => (0.0, 0.0, 0.0),
    }
}

fn cell_color_debug(cell: &Cell) -> (f32, f32, f32) {
    (cell.vx as f32 * 10.0, cell.vy as f32 * 10.0, (cell.dx + cell.dy) as f32 * 10.0)
}

fn cell_text_debug(cell: &Cell) -> String {
    let j = if cell.flags.contains(CellFlags::TRIED) { 'x' } else { ' ' };
    format!("{}{}{}\n{}{}{}", cell.vx, j, cell.vy, cell.dx, j, cell.dy)
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

fn print_frames(n: usize, cells: &mut Cells) {
    let mut frames: Vec<_> = (0..n).map(|_| {
        let frame = cells.format();
        // println!("{}", frame);
        cells.tick();
        frame.split('\n').map(String::from).collect::<Vec<_>>().into_iter()
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

fn random_signum(x: i32) -> i32 {
    if x != 0 {
        x.signum()
    } else if random() {
        1
    } else {
        -1
    }
}

fn next_pixel(dx: i32, dy: i32) -> (i32, i32) {
    if dy.abs() < dx.abs() {
        if (2 * dy).abs() - dx.abs() >= 0 {
            (dx.signum(), dy.signum())
        } else {
            (dx.signum(), 0)
        }
    } else if dy.abs() > dx.abs() {
        if (2 * dx).abs() - dy.abs() >= 0 {
            (dx.signum(), dy.signum())
        } else {
            (0, dy.signum())
        }
    } else {
        (dx.signum(), dy.signum())
    }
}

#[test]
fn test_next_pixel() {
    assert_eq!(next_pixel(-1, -1), (-1, -1));
    assert_eq!(next_pixel( 0, -1), ( 0, -1));
    assert_eq!(next_pixel( 1, -1), ( 1, -1));
    assert_eq!(next_pixel(-1,  1), (-1,  1));
    assert_eq!(next_pixel( 0,  1), ( 0,  1));
    assert_eq!(next_pixel( 1,  1), ( 1,  1));
    assert_eq!(next_pixel(-1,  0), (-1,  0));
    assert_eq!(next_pixel( 1,  0), ( 1,  0));

    assert_eq!(next_pixel(-1, -2), (-1, -1));
    assert_eq!(next_pixel( 1, -2), ( 1, -1));
    assert_eq!(next_pixel(-1,  2), (-1,  1));
    assert_eq!(next_pixel( 1,  2), ( 1,  1));

    assert_eq!(next_pixel(-2, -1), (-1, -1));
    assert_eq!(next_pixel(-2,  1), (-1,  1));
    assert_eq!(next_pixel( 2, -1), ( 1, -1));
    assert_eq!(next_pixel( 2,  1), ( 1,  1));

    assert_eq!(next_pixel(-2, -2), (-1, -1));
    assert_eq!(next_pixel(-2,  2), (-1,  1));
    assert_eq!(next_pixel( 2, -2), ( 1, -1));
    assert_eq!(next_pixel( 2,  2), ( 1,  1));
}