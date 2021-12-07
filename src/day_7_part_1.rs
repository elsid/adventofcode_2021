use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", align_crabs_position(std::io::stdin().lock()));
}

fn align_crabs_position(buffer: impl BufRead) -> i64 {
    let mut positions: Vec<i64> = buffer
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .split(',')
        .map(|v| i64::from_str(v).unwrap())
        .collect();
    positions.sort();
    let target = positions[positions.len() / 2];
    if positions.len() % 2 == 1 {
        return calculate_fuel_consumption(target, &positions);
    }
    calculate_fuel_consumption(target, &positions)
        .min(calculate_fuel_consumption(target + 1, &positions))
}

fn calculate_fuel_consumption(target: i64, positions: &[i64]) -> i64 {
    positions.iter().map(|v| (*v - target).abs()).sum()
}

#[test]
fn example_test() {
    let buffer = r#"16,1,2,0,4,2,7,1,2,14
"#
    .as_bytes();
    assert_eq!(align_crabs_position(buffer), 37);
}
