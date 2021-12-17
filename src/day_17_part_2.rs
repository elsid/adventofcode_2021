use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!(
        "{}",
        count_initial_velocities_to_hit(std::io::stdin().lock())
    );
    println!("SIMULATIONS={}", unsafe { SIMULATIONS });
}

fn count_initial_velocities_to_hit(buffer: impl BufRead) -> usize {
    let line = buffer.lines().next().unwrap().unwrap();
    let (x_area_str, y_area_str) = line.split_once(": ").unwrap().1.split_once(", ").unwrap();
    let (min_x, max_x) = parse_segment(x_area_str);
    let (min_y, max_y) = parse_segment(y_area_str);
    let target_area = Rect {
        min: Vec2 { x: min_x, y: min_y },
        max: Vec2 { x: max_x, y: max_y },
    };
    let initial_position = Vec2 { x: 0, y: 0 };
    let mut result = 0;
    for x in target_area.min.x.min(0)..=target_area.max.x.max(0) {
        for y in -target_area.max.x.abs()..=target_area.max.x.abs() {
            result += simulate_probe(initial_position, Vec2 { x, y }, &target_area) as usize;
        }
    }
    result
}

#[derive(Copy, Clone)]
struct Vec2 {
    x: i64,
    y: i64,
}

struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn contains_point(&self, point: &Vec2) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
    }
}

static mut SIMULATIONS: usize = 0;

fn simulate_probe(mut position: Vec2, mut velocity: Vec2, target: &Rect) -> bool {
    unsafe {
        SIMULATIONS += 1;
    }
    while can_reach_target_area(position, velocity, target) {
        if target.contains_point(&position) {
            return true;
        }
        position.x += velocity.x;
        position.y += velocity.y;
        match velocity.x.cmp(&0) {
            std::cmp::Ordering::Less => velocity.x += 1,
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => velocity.x -= 1,
        }
        velocity.y -= 1;
    }
    false
}

fn can_reach_target_area(position: Vec2, velocity: Vec2, target_area: &Rect) -> bool {
    match velocity.x.cmp(&0) {
        std::cmp::Ordering::Less => {
            if position.x < target_area.min.x {
                return false;
            }
        }
        std::cmp::Ordering::Equal => {
            if position.x < target_area.min.x || target_area.max.x < position.x {
                return false;
            }
        }
        std::cmp::Ordering::Greater => {
            if position.x > target_area.max.x {
                return false;
            }
        }
    }
    velocity.y >= 0 || position.y >= target_area.min.y
}

fn parse_segment(s: &str) -> (i64, i64) {
    let (min, max) = s.split_once('=').unwrap().1.split_once("..").unwrap();
    (i64::from_str(min).unwrap(), i64::from_str(max).unwrap())
}

#[test]
fn example_1_test() {
    let buffer = r#"target area: x=20..30, y=-10..-5
"#
    .as_bytes();
    assert_eq!(count_initial_velocities_to_hit(buffer), 112);
}

#[test]
fn example_2_test() {
    let buffer = r#"target area: x=-30..-20, y=-10..-5
"#
    .as_bytes();
    assert_eq!(count_initial_velocities_to_hit(buffer), 112);
}

#[test]
fn example_3_test() {
    let buffer = r#"target area: x=20..30, y=5..10
"#
    .as_bytes();
    assert_eq!(count_initial_velocities_to_hit(buffer), 103);
}

#[test]
fn example_4_test() {
    let buffer = r#"target area: x=-30..-20, y=5..10
"#
    .as_bytes();
    assert_eq!(count_initial_velocities_to_hit(buffer), 103);
}
