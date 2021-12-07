use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", align_crabs_position(std::io::stdin().lock()));
}

fn align_crabs_position(buffer: impl BufRead) -> i64 {
    let positions: Vec<i64> = buffer
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .split(',')
        .map(|v| i64::from_str(v).unwrap())
        .collect();
    find_optimal_fuel_consumption(
        *positions.iter().min().unwrap(),
        *positions.iter().max().unwrap(),
        &positions,
    )
}

fn find_optimal_fuel_consumption(min: i64, max: i64, positions: &[i64]) -> i64 {
    find_min(min, max, |v| calculate_fuel_consumption(v, &positions))
}

#[cfg(not(feature = "day_7_part_2_log"))]
fn find_min(min: i64, max: i64, f: impl Fn(i64) -> i64) -> i64 {
    (min..=max).map(f).min().unwrap()
}

#[cfg(feature = "day_7_part_2_log")]
fn find_min(mut min: i64, mut max: i64, f: impl Fn(i64) -> i64) -> i64 {
    let mut min_value = f(min).min(f(max));
    loop {
        let section = ((max - min) / 3).max(1);
        let left = min + section;
        let right = max - section;
        if left > right {
            break;
        }
        if left == right {
            min_value = f(left);
            break;
        }
        let left_value = f(left);
        let right_value = f(right);
        if left_value < right_value {
            max = right;
            min_value = left_value;
        } else {
            min = left;
            min_value = right_value;
        }
    }
    min_value
}

fn calculate_fuel_consumption(target: i64, positions: &[i64]) -> i64 {
    positions
        .iter()
        .map(|v| get_distance_fuel_consumption((*v - target).abs()))
        .sum()
}

fn get_distance_fuel_consumption(distance: i64) -> i64 {
    distance * (distance + 1) / 2
}

#[test]
fn example_test() {
    let buffer = r#"16,1,2,0,4,2,7,1,2,14
"#
    .as_bytes();
    assert_eq!(align_crabs_position(buffer), 168);
}
