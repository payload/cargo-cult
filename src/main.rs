
fn main() {
    println!("Hello, world!");
    let mut u = sands::Universe::new(200, 200);
    u.tick();
    u.tick();
    u.tick();
    u.tick();
}
