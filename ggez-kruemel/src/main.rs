// Copyright (c) 2020 Gilbert Röhrbein
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{Context, ContextBuilder, GameResult};

use rand::prelude::*;
use palette::*;

use nalgebra as na;
use nalgebra::{Vector2};

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
    
    debug_water(&mut cells);

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
        // println!("loop_count {} {}", self.loop_count, self.tick_n);
    }

    fn tick(&mut self) {
        let tick_n = self.tick_n;
        let w = self.w();
        let h = self.h();
        let bottom_to_top = (0..h).rev();
        let left_right = |x| if tick_n & 1 == 1 { w - x - 1 } else { x };

        for cell in self.cells.iter_mut() {
            cell.flags.remove(CellFlags::UPDATED);
            cell.flags.remove(CellFlags::TRIED);
            cell.flags.remove(CellFlags::V_FREE);
            cell.fx = 0;
            cell.fy = 1;
        }

        for y in 0..h {
            for x in 0..w {
                self.force_update(&self.cursor(x, y));
            }
        }

        for y in bottom_to_top {
            for x in 0..w {
                let x = left_right(x);
                let idx = self.idx(x, y);
                let cell = self.cells[idx];
                if cell.flags.contains(CellFlags::UPDATED) { continue }
                match cell.id {
                    Sand => self.update_sand(x, y, idx),
                    Water => self.water_update(&self.cursor(x, y)),
                    Wood =>self.update_wood(x, y, idx),
                    Special => self.special_update(&self.cursor(x, y)),
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

    fn cursor(&self, x: X, y: Y) -> CellCursor { CellCursor::new(x, y, self.w(), self.h()) }
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
            if !self.cells[idx].flags.contains(CellFlags::UPDATED) {
                return self.cells[idx];
            } 
        }
        Cell::unavailable()
    }

    #[inline(always)]
    fn update_sand_acceleration(&mut self, x: X, y: Y, mut cell: Cell) -> Cell {
        let gy = 1;
        let vx = cell.vx();
        let vy = cell.vy();
        let (h, v) = next_pixel(vx, vy + gy);

        let d = self.cell(x + h, y + v);
        
        if let Some(poo) = self.checked_idx(x + h, y + v) {
            self.cells[poo].flags.insert(CellFlags::V_FREE);
        }

        let accel = gy as i8;
        if d.id == Empty {
            cell.vy = cell.vy.saturating_add(accel);
        } else if d.id == Sand {
            if d.vy > cell.vy {
                cell.vy = cell.vy.saturating_add(d.vy.saturating_sub(cell.vy).min(accel));
            } else
            // If sand stands still v=0, try to ripple down artificially.
            // Alternative idea:
            //  If sand stands perfectly still v=0 d=0, don't ripple down.
            //  But when sand moves, apply at least d=1 to surrounding sand.
            if vx == 0 && vy == 0 {
                let empty_l = self.cell(x - 1, y + gy).id == Empty;
                let empty_r = self.cell(x + 1, y + gy).id == Empty;
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
            if (accel > 0 && cell.vy < accel) || (accel < 0 && cell.vy > accel) {
                cell.vy += accel;
            }
        } else {
            cell.vx -= cell.vx.signum();
            cell.vy -= cell.vy.signum();
        }

        cell
    }

    fn update_sand(&mut self, x: X, y: Y, idx: usize) {
        let mut cell: Cell = self.cells[idx];
        assert_eq!(cell.id, Sand);
        
        cell = self.update_sand_acceleration(x, y, cell);
        
        let mut dx = cell.dx.saturating_add(cell.vx);
        let mut dy = cell.dy.saturating_add(cell.vy);
        
        let mut loop_count = 0;
        let mut cursor0 = self.cursor(x, y);
        while dx >= 10 || dy >= 10 || dx <= -10 || dy <= -10 {
            loop_count += 1;
            // TODO this can be done in a single line
            let (h, v) = next_pixel(dx as i32 / 10, dy as i32 / 10);
            let mut cursor1 = cursor0.add(h, v);
            let next = self.cell(cursor1.x, cursor1.y);

            if next.id == Empty {
                dx -= 10 * h as i8;
                dy -= 10 * v as i8;
                self.cells[cursor0.idx] = self.cells[cursor1.idx];
                cursor0 = cursor1;
            } else {
                self.update_sand_deflection(&mut cursor0, &mut cursor1, &mut dx, &mut dy, h as i8, v as i8);
            }
        }

        cell.dx = dx;
        cell.dy = dy;
        self.cells[cursor0.idx] = cell;
        self.loop_count = self.loop_count.max(loop_count);
    }

    fn update_sand_deflection(&mut self, cursor0: &mut CellCursor, cursor1: &mut CellCursor, dx: &mut i8, dy: &mut i8, h: i8, v: i8) {
        let mut cell = self.cell_from_cursor(&cursor0);
        let next = deflect(cursor1, &v8(h, v), self, &mut cell);
        let next = cursor0.add(next.x, next.y);
        self.cells[cursor0.idx] = self.cells[next.idx];
        *cursor0 = next;
        *dx = cell.dx;
        *dy = cell.dy;
    }

    fn special_update(&mut self, cursor: &CellCursor) {
        // let left = cursor.add(-1, 0);
        // let mut cell = self.cell_checked(&left);
        // cell.vx = cell.vx.saturating_sub(1);
        // self.cells[left.idx] = cell;

        self.cells[cursor.idx].vx -= 20;
        self.cells[cursor.idx].id = Water;
    }

    fn update_wood(&mut self, _x: X, _y: Y, _idx: usize) {
    }

    //
    //

    fn water_update(&mut self, cursor: &CellCursor) {
        let mut cell = self.cells[cursor.idx];
        assert!(cell.id == Water);

        if cell.vy < 1
            || self.cell_is(&cursor.add(0, 1), Empty)
            || self.cell_is(&cursor.add(-1, 1), Empty)
            || self.cell_is(&cursor.add(1, 1), Empty)
            || self.cell_is(&cursor.add(-1, 0), Empty)
            || self.cell_is(&cursor.add(1, 0), Empty)
            {
            cell.vy = cell.vy.saturating_add(1);
        }

        let mut dx = cell.dx.saturating_add(cell.vx);
        let mut dy = cell.dy.saturating_add(cell.vy);

        let mut cursor0 = cursor.clone();
        let mut next_overwrite: Option<CellCursor> = None;
        
        let mut loop_counter = 0;
        while dx >= 10 || dy >= 10 || dx <= -10 || dy <= -10 {
            loop_counter += 1;
            if loop_counter > 100 {
                dbg!("potential endless loop detected");
            }

            // TODO this can be done in a single line
            let (h, v, next, cursor1) = if let Some(n) = next_overwrite {
                next_overwrite = None;
                let h = n.x - cursor0.x;
                let v = n.y - cursor0.y;
                (h as i8, v as i8, self.cell_checked(&n), n)
            } else {
                let (h, v) = next_pixel(dx as i32 / 10, dy as i32 / 10);
                let cursor1 = cursor0.add(h, v);
                let next = self.cell_checked(&cursor1);
                (h as i8, v as i8, next, cursor1)
            };

            if next.id == Empty {
                dx -= 10 * h.abs() * dx.signum();
                dy -= 10 * v.abs() * dy.signum();
                self.cells[cursor0.idx] = self.cells[cursor1.idx];
                cursor0 = cursor1;
            } else {
                cell.dx = dx;
                cell.dy = dy;

                if let Some(new_dir) = deflect2(&cursor1, &v8(h, v), self) {
                    cell.consume_energy(&new_dir);

                    cell.vx -= cell.vx.signum();
                    cell.vy -= cell.vy.signum();

                    cell.change_dir(&new_dir);
                    let next = cursor0.add(new_dir.x, new_dir.y);
                    self.cells[cursor0.idx] = self.cells[next.idx];
                    cursor0 = next;
                } else if let Some(new_dir) = deflect2(&cursor0, &v8(h, v), self) {
                    cell.consume_energy(&new_dir);
                    
                    cell.vx -= cell.vx.signum();
                    cell.vy -= cell.vy.signum();
                    
                    let new_dir = v32(new_dir.x - h as i32, new_dir.y - v as i32);
                    cell.change_dir(&new_dir);
                    let next = cursor0.add(new_dir.x, new_dir.y);
                    self.cells[cursor0.idx] = self.cells[next.idx];
                    cursor0 = next;
                } else {
                    cell.stop();
                }

                dx = cell.dx;
                dy = cell.dy;
            }
        }

        // if cell.vy < 1 || self.cell_is(&cursor.add(0, 1), Empty) {
        //     cell.vy = cell.vy.saturating_add(1);
        // }

        /*
        let lcell = self.cell_checked(&cursor.add(-1,  0));
        let rcell = self.cell_checked(&cursor.add( 1,  0));
        let ucell = self.cell_checked(&cursor.add( 0, -1));
        let dcell = self.cell_checked(&cursor.add( 0,  1));

        if rcell.vx < -12 {
            print!("");
        }

        if lcell.id != Empty { cell.vx = cell.vx.saturating_add(lcell.vx.saturating_sub(cell.vx).signum()); }
        if rcell.id != Empty { cell.vx = cell.vx + (rcell.vx - cell.vx).signum(); }
        if ucell.id != Empty { cell.vx = cell.vx.saturating_add(ucell.vx.saturating_sub(cell.vx).signum()); }
        if dcell.id != Empty { cell.vx = cell.vx.saturating_add(dcell.vx.saturating_sub(cell.vx).signum()); }
        if lcell.id != Empty { cell.vy = cell.vy.saturating_add(lcell.vy.saturating_sub(cell.vy).signum()); }
        if rcell.id != Empty { cell.vy = cell.vy.saturating_add(rcell.vy.saturating_sub(cell.vy).signum()); }
        if ucell.id != Empty { cell.vy = cell.vy.saturating_add(ucell.vy.saturating_sub(cell.vy).signum()); }
        if dcell.id != Empty { cell.vy = cell.vy.saturating_add(dcell.vy.saturating_sub(cell.vy).signum()); }
        */

        cell.dx = dx;
        cell.dy = dy;
        self.cells[cursor0.idx] = cell;
    }

    fn force_update(&mut self, cursor: &CellCursor) {
        let mut cell = self.cell_from_cursor(cursor);
        let gravity = 1;
        cell.fx = cell.vx;
        cell.fy = cell.vy + gravity;
        //self.cells[cursor.idx] = cell;
    }

    fn cell_from_cursor(&self, cursor: &CellCursor) -> Cell {
        if cursor.in_bounds() {
            let cell = self.cells[cursor.idx];
            if !cell.flags.contains(CellFlags::UPDATED) {
                return cell;
            } 
        }
        Cell::unavailable()
    }

    fn cell_checked(&self, cursor: &CellCursor) -> Cell {
        if cursor.in_bounds() {
            return self.cells[cursor.idx];
        }
        Cell::unavailable()
    }

    fn cell_is(&self, cursor: &CellCursor, id: CellId) -> bool {
        self.in_bounds(cursor.x, cursor.y) && self.cells[cursor.idx].id == id
    }

    fn cell_mass(&self, cursor: &CellCursor) -> f32 {
        let id = if self.in_bounds(cursor.x, cursor.y) { self.cells[cursor.idx].id } else { Unavailable };
        match id {
            Empty => 0.0,
            Water => 1.0,
            Sand => 2.0,
            Wood => 0.9,
            Special => 2.0,
            Unavailable => 100.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    id: CellId,
    vx: i8,
    vy: i8,
    dx: i8,
    dy: i8,
    fx: i8,
    fy: i8,
    random: f32,
    flags: CellFlags,

    rho: f32,
}

bitflags! {
    struct CellFlags: u8 {
        const UPDATED = 0b00000001;
        const TRIED   = 0b00000010;
        const V_FREE  = 0b00000100;
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Cell {
    fn new(id: CellId) -> Self { Self { vx: 0, vy: 0, dx: 0, dy: 0, fx: 0, fy: 0, random: random(), flags: CellFlags::empty(), id, rho: std::f32::NAN } }
    fn empty() -> Self { Self { ..Self::new(Empty) } }
    fn sand() -> Self { Self::new(Sand) }
    fn water() -> Self { Self::new(Water) }
    fn wood() -> Self { Self::new(Wood) }
    fn special() -> Self { Self::new(Special) }
    fn unavailable() -> Self { Self::new(Unavailable) }

    fn char(&self) -> char {
        match self.id {
            Empty => ' ',
            Sand => '.',
            Water => '~',
            Wood => '#',
            Special => '*',
            Unavailable => 'X',
        }
    }

    fn vx(&self) -> i32 { self.vx as i32 }
    fn vy(&self) -> i32 { self.vy as i32 }
    fn set_dx(&mut self, v: i32) { self.dx = v as i8; }
    fn set_dy(&mut self, v: i32) { self.dy = v as i8; }

    fn is_something(&self) -> bool { self.id != Unavailable && self.id != Empty }
}

impl From<CellId> for Cell {
    fn from(id: CellId) -> Cell {
        match id {
            Empty => Cell::empty(),
            Sand => Cell::sand(),
            Water => Cell::water(),
            Wood => Cell::wood(),
            Special => Cell::special(),
            Unavailable => Cell::unavailable(),
        }
    }
}

//#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellId {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Wood = 3,
    Special = 254,
    Unavailable = 255,
}

struct MyGame {
    debug_cells: bool,
    cells: Cells,

    paint_primary_id: CellId,
    paint_secondary_id: CellId,
    paint_size: u32,
    scale: u32,
    
    paused: bool,
}

impl MyGame {
    fn new(ctx: &mut Context) -> MyGame {
        let debug_cells = false;
        let scale = 4;
        let mut game = MyGame {
            debug_cells,
            cells: Self::create_cells(debug_cells, scale as usize, ctx),
            paint_primary_id: Sand,
            paint_secondary_id: Empty,
            paint_size: 4,
            scale,
            paused: true,
        };

        debug_water(&mut game.cells);
        game
    }

    fn switch_cells(&mut self, ctx: &mut Context) {
        self.debug_cells = !self.debug_cells;
        self.cells = Self::create_cells(self.debug_cells, self.scale as usize, ctx);
        debug_water(&mut self.cells);
    }

    fn create_cells(debug_cells: bool, scale: usize, ctx: &mut Context) -> Cells {
        if debug_cells {
            Cells::new(11, 11)
        } else {
            let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
            let w = size.width as usize / scale;
            let h = size.height as usize / scale;
            Cells::new(w, h)
        }
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
            D => self.switch_cells(ctx),

            Minus => self.scale = 1.max(self.scale - 1),
            Equals if shift => self.scale = self.scale + 1,
            Add => self.scale = self.scale + 1,
            Key0 => self.scale = 8,

            Key1 if !shift => self.paint_primary_id = Empty,
            Key2 if !shift => self.paint_primary_id = Sand,
            Key3 if !shift => self.paint_primary_id = Water,
            Key4 if !shift => self.paint_primary_id = Wood,
            Key5 if !shift => self.paint_primary_id = Special,
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

        if let Ok(mesh) = builder.build(ctx) {
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        if false {
            for y in 0..=self.cells.h() {
                for x in 0..=self.cells.w() {
                    self.cell_debug(ctx, x as f32, y as f32, &self.cells.cell(x, y));
                }
            }
        }

        Ok(())
    }

    fn cell_debug(&mut self, ctx: &mut Context, x: f32, y: f32, cell: &Cell) {
        let s = self.scale as f32;
        let w = self.cells.w() as f32;
        let params = graphics::DrawParam::new()
            .color(graphics::BLACK)
            .dest([x * s, y * s])
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
        _ => (0.9, 0.9, 0.9),
    }
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

#[derive(Debug, Copy, Clone)]
struct CellCursor {
    x: X,
    y: Y,
    w: X,
    h: Y,
    idx: usize,
}

impl CellCursor {
    fn new(x: X, y: Y, w: X, h: Y) -> Self {
        Self { x, y, w, h, idx: (y * w + x) as usize }
    }

    fn add(&self, dx: X, dy: Y) -> Self {
        Self::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    fn in_bounds(&self) -> bool {
        self.x >= 0 && self.x < self.w && self.y >= 0 && self.y < self.h
    }
}

fn choose_direction_factor(x: i32, l: bool, r: bool) -> i32 {
    match (l, r) {
        (false, false) => 0,
        (true, false) => -1,
        (false, true) => 1,
        (true, true) => random_signum(x),
    }
}

fn debug_water(cells: &mut Cells) {
    let w2 = cells.w() / 2;
    let h = cells.h() - 1;

    for x in w2-2..=w2+2 {
        for y in h-5..=h {
            if y != h {
                cells.paint(x, y, Water);
            } else {
                cells.paint(x, y, Wood);
            }
        }
    }
}

fn dx_diff(a: Cell, b: Cell) -> i8 {
    if a.id == Unavailable || a.id == Empty { 0 } else { a.dx - b.dx }
}

fn dy_diff(a: Cell, b: Cell) -> i8 {
    if a.id == Unavailable || a.id == Empty {
        0
    } else {
        a.dy - b.dy
    }
}

type V = Vector2<i32>;

fn v<N: Into<i32>>(x: N, y: N) -> Vector2<i32> { V::new(x.into(), y.into()) }
fn v32(x: i32, y: i32) -> V { V::new(x.into(), y.into()) }
fn v8(x: i8, y: i8) -> Vector2<i32> { V::new(x.into(), y.into()) }

fn orthogonal_offsets(dir: &V) -> [Option<V>; 2] {
    match (dir.x == 0, dir.y == 0) {
        (true, false) => [Some(v(1, 0)), Some(v(-1, 0))],
        (false, true) => [Some(v(0, 1)), Some(v(0, -1))],
        (true, true) => [Some(v(-dir.x, 0)), Some(v(0, -dir.y))],
        (false, false) => [None, None],
    }
}

fn deflect(cursor: &CellCursor, dir: &V, cells: &Cells, cell: &mut Cell) -> V { 
    let choice =
        orthogonal_offsets(dir)
        .iter()
        .cloned()
        .filter_map(|o| o)
        .map(|o| (o, cursor.add(o.x, o.y)))
        .map(|c| (c.0, c.1, cells.cell_checked(&c.1)))
        .filter(|c| c.2.id == Empty)
        .choose(&mut rand::thread_rng());
    if let Some(choice) = choice {
        let new_dir = dir + choice.0;
        cell.consume_energy(dir);
        cell.change_dir(&new_dir);
        new_dir
    } else {
        cell.stop();
        v(0, 0)
    }
}

fn deflect2(cursor: &CellCursor, dir: &V, cells: &Cells) -> Option<V> { 
    orthogonal_offsets(dir)
        .iter()
        .cloned()
        .filter_map(|o| o)
        .map(|o| (o, cursor.add(o.x, o.y)))
        .map(|c| (c.0, c.1, cells.cell_checked(&c.1)))
        .filter(|c| c.2.id == Empty)
        .choose(&mut rand::thread_rng())
        .map(|choice| dir + choice.0)
}

impl Cell {
    fn consume_energy(&mut self, dir: &V) {
        self.dx -= self.dx.signum() * dir.x.abs() as i8 * 10;
        self.dy -= self.dy.signum() * dir.y.abs() as i8 * 10;
    }

    fn change_dir(&mut self, dir: &V) {
        let dirsum = dir.x.signum().abs() + dir.y.signum().abs();

        let dx = self.dx as i32;
        let dy = self.dy as i32;
        let dsum = dx.abs() + dy.abs();
        self.dx = (dir.x * dsum / dirsum) as i8;
        self.dy = (dir.y * (dsum - self.dx as i32))  as i8;
        
        // let vx = self.vx as i32;
        // let vy = self.vy as i32;
        // let vsum = vx.abs() + vy.abs();
        // self.vx = self.vx.abs() * dir.x.signum();
        // self.vy = self.vy.abs() * dir.y.signum();
    }

    fn stop(&mut self) {
        self.dx = 0;
        self.dy = 0;
        self.vx = 0;
        self.vy = 0;
    }
}

fn cursors_to_neighbors(cursor: &CellCursor) -> [CellCursor; 8] {
    [
        cursor.add(-1, -1),
        cursor.add(0, -1),
        cursor.add(1, -1),
        cursor.add(-1, 0),
        cursor.add(1, 0),
        cursor.add(-1, 1),
        cursor.add(0, 1),
        cursor.add(1, 1),
    ]
}