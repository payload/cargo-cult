// Copyright (c) 2020 Max Bittker, Gilbert Röhrbein
use rand::{thread_rng, Rng};

pub fn rand_int(n: i32) -> i32 {
    thread_rng().gen_range(0, n)
}

pub fn rand_bool(p: f64) -> bool {
    thread_rng().gen_bool(p)
}

pub fn rand_1() -> f64 {
    thread_rng().gen()
}

pub fn rand_low100() -> i32 {
    (rand_1() * rand_1() * 100.) as i32
}

pub fn rand_dir() -> i32 {
    let i = rand_int(1000);
    (i % 3) - 1
}
pub fn rand_dir_2() -> i32 {
    let i = rand_int(1000);
    if (i % 2) == 0 {
        -1
    } else {
        1
    }
}

pub fn rand_vec() -> (i32, i32) {
    let i = rand_int(2000);
    match i % 9 {
        0 => (1, 1),
        1 => (1, 0),
        2 => (1, -1),
        3 => (0, -1),
        4 => (-1, -1),
        5 => (-1, 0),
        6 => (-1, 1),
        7 => (0, 1),
        _ => (0, 0),
    }
}

pub fn rand_vec_8() -> (i32, i32) {
    let i = rand_int(2000);
    match i % 8 {
        0 => (1, 1),
        1 => (1, 0),
        2 => (1, -1),
        3 => (0, -1),
        4 => (-1, -1),
        5 => (-1, 0),
        6 => (-1, 1),
        _ => (0, 1),
    }
}
pub fn adjacency_right(dir: (i32, i32)) -> (i32, i32) {
    match dir {
        (0, 1) => (1, 1),
        (1, 1) => (1, 0),
        (1, 0) => (1, -1),
        (1, -1) => (0, -1),
        (0, -1) => (-1, -1),
        (-1, -1) => (-1, 0),
        (-1, 0) => (-1, 1),
        (-1, 1) => (0, 1),
        _ => (0, 0),
    }
}
pub fn adjacency_left(dir: (i32, i32)) -> (i32, i32) {
    match dir {
        (0, 1) => (-1, 1),
        (1, 1) => (0, 1),
        (1, 0) => (1, 1),
        (1, -1) => (1, 0),
        (0, -1) => (1, -1),
        (-1, -1) => (0, -1),
        (-1, 0) => (-1, -1),
        (-1, 1) => (-1, 0),
        _ => (0, 0),
    }
}
pub fn join_dy_dx(dx: i32, dy: i32) -> u8 {
    (((dx + 1) * 3) + (dy + 1)) as u8
}

pub fn split_dy_dx(s: u8) -> (i32, i32) {
    let s: i32 = s as i32;

    let dx: i32 = (s / 3) - 1;

    let dy: i32 = (s % 3) - 1;

    (dx, dy)
}
