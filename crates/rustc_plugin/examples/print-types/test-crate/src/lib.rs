pub struct Point {
  pub x: f64,
  pub y: f64,
}

pub enum Color {
  Red,
  Green,
  Blue,
}

pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

pub fn distance(p: Point) -> f64 {
  (p.x * p.x + p.y * p.y).sqrt()
}

pub fn describe(c: Color) -> &'static str {
  match c {
    Color::Red => "red",
    Color::Green => "green",
    Color::Blue => "blue",
  }
}
