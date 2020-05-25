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
        //println!("loop_count {}", self.loop_count);
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
        }

        for y in bottom_to_top {
            for x in 0..w {
                let x = left_right(x);
                let idx = self.idx(x, y);
                let cell = self.cells[idx];
                if cell.flags.contains(CellFlags::UPDATED) { continue }
                match cell.id {
                    Sand => self.update_sand(x, y, idx),
                    Water => self.water_update(x, y, idx),
                    Wood =>self.update_wood(x, y, idx),
                    Special => self.update_special(x, y, idx),
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

    fn cursor(&self, x: X, y: Y) -> CellCursor { CellCursor::new(x, y, self.w()) }
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
        // TODO add impact energy transfer
        // TODO could increase v too
        // TODO unify code paths if possible
        // self.cells[cursor.idx].flags.insert(CellFlags::TRIED);
        let path;
        let dsum0 = dx.abs() as i32 + dy.abs() as i32;

        if v != 0 && h == 0 {
            path = 1;
            let dr_empty = self.cell(cursor1.x + 1, cursor1.y).id == Empty;
            let dl_empty = self.cell(cursor1.x - 1, cursor1.y).id == Empty;
            let dir = choose_direction_factor(*dx as i32, dl_empty, dr_empty) as i8;
            let low_mid = (dy.abs() - dx.abs()) / 2;
            *dx = dir * (dy.abs() - low_mid);
            *dy = dy.signum() * low_mid;
        } else if v != 0 && h != 0 {
            let h_empty = self.cell(cursor1.x + h as i32, cursor1.y).id == Empty;
            let v_empty = self.cell(cursor1.x, cursor1.y + v as i32).id == Empty;

            if (h_empty && !v_empty) || (h_empty && v_empty && random()) {
                path = 2;
                *dx -= 10 * h;
                *dy -= 10 * v;
                *cursor1 = cursor1.add(h as i32, 0);
                self.cells[cursor0.idx] = self.cells[cursor1.idx];
            } else if v_empty {
                path = 3;
                *dx -= 10 * h;
                *dy -= 10 * v;
                *cursor1 = cursor1.add(0, v as i32);
                self.cells[cursor0.idx] = self.cells[cursor1.idx];
            } else {
                path = 4;
                *dx = 0;
                *dy = 0;
            }
        } else if v == 0 && h != 0 {
            path = 5;
            let d_empty = self.cell(cursor1.x, cursor1.y + 1).id == Empty;
            let u_empty = self.cell(cursor1.x, cursor1.y - 1).id == Empty;
            let dir = choose_direction_factor(*dy as i32, u_empty, d_empty) as i8;
            let low_mid = (dx.abs() - dy.abs()) / 2;
            let xdx = dx.signum() * low_mid;
            let xdy = dir * (dx.abs() - low_mid);
            *dx = xdx;
            *dy = xdy;
        } else {
            path = 6;
            *dx = 0;
            *dy = 0;
        }

        let dsum1 = dx.abs() as i32 + dy.abs() as i32;
        if dsum1 > dsum0 {
            println!("created energy from nothing: path={} diff={}", path, dsum1 - dsum0);
        }
    }

    fn update_special(&mut self, x: X, y: Y, _idx: usize) {
        let t = (self.tick_n as f32) / 30.0;
        let sx = t.cos() * 2.0;
        let sy = t.sin() * 2.0;
        if let Some(idx) = self.checked_idx(x + sx as i32, y + sy as i32) {
            if self.cells[idx].id == Empty {
                let mut cell = Cell::sand();
                cell.vx = (sx * 4.0) as i8;
                cell.vy = (sy * if sy < 0.0 { 12.0 } else { 4.0 }) as i8;
                cell.flags.insert(CellFlags::UPDATED);
                self.cells[idx] = cell;
            }
        }
    }

    fn update_wood(&mut self, _x: X, _y: Y, _idx: usize) {
    }

    //
    //

    fn water_update(&mut self, x: X, y: Y, idx: usize) {
        let mut cursor0 = self.cursor(x, y);
        let mut cell = self.cells[cursor0.idx];
        assert!(cell.id == Water);

        

        self.cells[cursor0.idx] = cell;
    }

    #[inline(always)]
    fn water_acceleration(&mut self, x: X, y: Y, mut cell: Cell) -> Cell {
        let gy = 1;
        let vx = cell.vx();
        let vy = cell.vy();
        let (h, v) = next_pixel(vx, vy + gy);
        let cursor1 = self.cursor(x + h, y + v);
        let mut d = self.cell(cursor1.x, cursor1.y);
        
        if let Some(poo) = self.checked_idx(x + h, y + v) {
            self.cells[poo].flags.insert(CellFlags::V_FREE);
        }

        let accel = gy as i8;
        if d.id == Empty {
            cell.vy = cell.vy.saturating_add(accel);
        } else if d.vy > cell.vy {
            cell.vy = cell.vy.saturating_add(d.vy.saturating_sub(cell.vy).min(accel));
        } else if d.id == Water {
            d.vx = d.vx.saturating_add(h as i8);
            d.vy = d.vy.saturating_add(v as i8);
            self.cells[cursor1.idx] = d;
        } else {
            cell.vx -= cell.vx.signum();
            cell.vy -= cell.vy.signum();
        }

        cell
    }

    fn cell_is(&self, cursor: &CellCursor, id: CellId) -> bool {
        self.in_bounds(cursor.x, cursor.y) && self.cells[cursor.idx].id == id
    }

    fn cell_mass(&self, cursor: &CellCursor) -> f32 {
        let id = if self.in_bounds(cursor.x, cursor.y) { self.cells[cursor.idx].id } else { Unavailable };
        match id {
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
    vx: i8,
    vy: i8,
    dx: i8,
    dy: i8,
    random: f32,
    flags: CellFlags,
    id: CellId,

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
    fn new(id: CellId) -> Self { Self { vx: 0, vy: 0, dx: 0, dy: 0, random: random(), flags: CellFlags::empty(), id, rho: std::f32::NAN } }
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

    fn rho(&mut self, cursor: &CellCursor, cells: &Cells) -> f32 {
        if self.rho.is_nan() {
            self.rho = calc_density_contribution(cursor, cells);      
        }
        self.rho
    }
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

#[repr(u8)]
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
    particles: Particles,

    paint_primary_id: CellId,
    paint_secondary_id: CellId,
    paint_size: u32,
    scale: u32,
    
    paused: bool,
}

impl MyGame {
    fn new(ctx: &mut Context) -> MyGame {
        let debug_cells = false;
        let scale = 8;
        let mut game = MyGame {
            debug_cells,
            cells: Self::create_cells(debug_cells, scale as usize, ctx),
            particles: Self::create_particles(debug_cells, scale as usize, ctx),
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

    fn create_particles(debug_cells: bool, scale: usize, ctx: &mut Context) -> Particles {
        if debug_cells {
            Particles::new(11, 11)
        } else {
            let size = ggez::graphics::window(ctx).get_inner_size().unwrap();
            let w = size.width as usize / scale;
            let h = size.height as usize / scale;
            Particles::new(w, h)
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
            if self.paint_primary_id == Water {
                for_circle(self.paint_size, &mut |dx, dy| {
                    self.particles.particles.push(Particle::new((x + dx) as f32, (y + dy) as f32));
                });
            } else {
                for_circle(self.paint_size, &mut |dx, dy| {
                    if rand(0.8) {
                        self.cells.paint(x + dx, y + dy, self.paint_primary_id);
                    }
                });
            }
        } else if button_pressed(ctx, MouseButton::Right) {
            for_circle(self.paint_size, &mut |dx, dy| {
                if rand(0.8) {
                    self.cells.paint(x + dx, y + dy, self.paint_secondary_id);
                }
            });
        }

        if !self.paused {
            self.cells.sim_update();
            self.particles.tick(&self.cells);
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

        for p in self.particles.particles.iter() {
            draw(p.x(), p.y(), (0.0, 0.0, 1.0));
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
    idx: usize,
}

impl CellCursor {
    fn new(x: X, y: Y, w: X) -> Self {
        Self { x, y, w, idx: (y * w + x) as usize }
    }

    fn add(&self, dx: X, dy: Y) -> Self {
        Self::new(self.x + dx, self.y + dy, self.w)
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
    /*
    let rand = |p| random::<f64>() < p;
    for_circle(self.paint_size, &mut |dx, dy| {
        if rand(0.8) {
            self.cells.paint(self.cells.w()/2 -1 + dx, 5 + dy, Water);
        }
    });
    */

    let w2 = cells.w() / 2;
    cells.paint(w2-1, 10, Water);
    cells.paint(w2-1, 9, Water);
    cells.paint(w2-1, 8, Water);
    cells.paint(w2, 10, Water);
    cells.paint(w2, 9, Water);
    cells.paint(w2, 8, Water);
    cells.paint(w2, 7, Water);
    cells.paint(w2, 6, Water);
    cells.paint(w2, 5, Water);
    cells.paint(w2+1, 10, Water);
    cells.paint(w2+1, 9, Water);
    cells.paint(w2+1, 8, Water);
}

struct Particles {
    particles: Vec<Particle>,
    bounds: Bounds,
}

#[derive(Debug)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    fx: f32,
    fy: f32,
    rho: f32,
    p: f32,
}

impl Particles {
    fn new(w: usize, h: usize) -> Particles {
        Self {
            particles: Vec::new(),
            bounds: Bounds { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }
        }
    }

    fn add_dam(mut self) -> Self {
        let bounds = Bounds {
            x: self.bounds.x + 0.4 * self.bounds.w,
            y: self.bounds.y + 0.6 * self.bounds.h,
            w: self.bounds.w * 0.2,
            h: self.bounds.h * 0.4,
        };
        for iy in 0..bounds.w as i32 {
            for ix in 0..bounds.h as i32 {
                let x: f32 = 0.2 * random::<f32>() + bounds.x + ix as f32;
                let y: f32 = 0.2 * random::<f32>() + bounds.y + iy as f32;
                if bounds.inside(x, y) {
                    self.particles.push(Particle::new(x, y));
                }
            }
        }
        self
    }

    fn tick(&mut self, cells: &Cells) {
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate(cells);
    }

    fn integrate(&mut self, cells: &Cells) {
        for p in self.particles.iter_mut() {
            println!("{:?}", p);
            let ff = DT / p.rho;
            p.vx += ff * p.fx;
            p.vy += ff * p.fy;
            p.x += (DT * p.vx).max(-30.0).min(30.0);
            p.y += (DT * p.vy).max(-30.0).min(30.0);

            let cursor = cells.cursor(p.x(), p.y());

            if !cells.cell_is(&cursor, Empty) {
                let mut d = 1;
                loop {
                    let mut empties = Vec::with_capacity(128);
                    for i in -d..=d {
                        let c = cursor.add(i, -d);
                        if cells.cell_is(&c, Empty) { empties.push(c); }
                        let c = cursor.add(i,  d);
                        if cells.cell_is(&c, Empty) { empties.push(c); }
                        let c = cursor.add(-d, i);
                        if cells.cell_is(&c, Empty) { empties.push(c); }
                        let c = cursor.add( d, i);
                        if cells.cell_is(&c, Empty) { empties.push(c); }
                    }

                    if empties.is_empty() {
                        d += 1;
                    } else {
                        let empty: CellCursor = empties[random::<usize>() % empties.len()];
                        p.x = empty.x as f32;
                        p.y = empty.y as f32;
                        break;
                    }
                }
                println!("d {} {} {}", d, cursor.x, cursor.y);

                p.vx *= BOUND_DAMPING;
                p.vy *= BOUND_DAMPING;
            }
        }
    }

    fn compute_density_pressure(&mut self) {
        let len = self.particles.len();
        for i in 0..len {
            let mut rho = 0.0;
            let pi = &self.particles[i];
            for j in 0..len {
                let pj = &self.particles[j];
                let dx = pj.x - pi.x;
                let dy = pj.y - pi.y;
                let near = H_SQUARE - dx*dx - dy*dy;
                if (dx <= 1.0 && dx >= -1.0) || (dy <= 1.0 && dy >= -1.0) {
                    rho += MASS * POLYG * near * near * near;
                }
            }
            let pi = &mut self.particles[i];
            pi.rho = rho;
            pi.p = GAS_CONST * (rho - REST_DENS);
            println!("rho {}", rho);
        }
    }

    fn compute_forces(&mut self) {
        let len = self.particles.len();
        for i in 0..len {
            let pi = &self.particles[i];

            let mut fpx = 0.0;
            let mut fpy = 0.0;
            let mut fvx = 0.0;
            let mut fvy = 0.0;

            for j in 0..len {
                let pj = &self.particles[j];

                if i == j {
                    continue;
                }

                let dx = pj.x - pi.x;
                let dy = pj.y - pi.y;
                let dist =  (dx*dx + dy*dy).sqrt();
                let nx = dx / dist;
                let ny = dy / dist;
                let near = H - dist;
                let pressure = MASS * (pi.p + pj.p) / (2.0 * pj.rho) * SPIKY_GRAD * near * near;
                let viscosity = VISC * MASS * 1.0 / pj.rho * VISC_LAP * near;

                if near > 0.0 {
                    fpx -= nx * pressure;
                    fpy -= ny * pressure;
                    fvx += (pj.vx - pi.vx) * viscosity;
                    fvy += (pj.vy - pi.vy) * viscosity;
                }
            }

            let pi = &mut self.particles[i];
            pi.fx = fpx + fvx;
            pi.fy = fpy + fvy + G * pi.rho;
        }
    }
}

impl Particle {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y, vx: 0.0, vy: 0.0, fx: 0.0, fy: 0.0, rho: 0.0, p: 0.0 }
    }

    fn x(&self) -> i32 { self.x.min(512.0).max(-512.0) as i32 }
    fn y(&self) -> i32 { self.y.min(512.0).max(-512.0) as i32 }
}

struct Bounds {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Bounds {
    fn inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
    fn r(&self) -> f32 { self.x + self.w }
    fn b(&self) -> f32 { self.y + self.h }
}

const G: f32 = 12000.0 * 9.8;
const H: f32 = 2.0;
const H_SQUARE: f32 = H * H;
const MASS: f32 = 1.0;
const VISC: f32 = 0.0;
const REST_DENS: f32 = 1.0;
const GAS_CONST: f32 = 1.0;
const DT: f32 = 0.0008;
const BOUND_DAMPING: f32 = -0.5;

const POLYG: f32 = 1.0 / H;
const SPIKY_GRAD: f32 = 1.0 / H;
const VISC_LAP: f32 = 1.0 / H;
// const POLYG: f32 = 315.0 / (65.0 * 3.14159 * H*H*H*H*H*H*H*H*H);
// const SPIKY_GRAD: f32 = -45.0 / (3.14159 * H*H*H*H*H*H);
// const VISC_LAP: f32 = 45.0 / (3.14159 * H*H*H*H*H*H);

fn calc_density_contribution(cursor: &CellCursor, cells: &Cells) -> f32 {
    let mut rho = 0.0;
    let other = cursor.add(-1, -1);
    let f = 1;
    let mass = cells.cell_mass(&other);
    rho
}