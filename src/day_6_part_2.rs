use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!(
        "{}",
        simulate_lanternfish_population(std::io::stdin().lock())
    );
}

const MAX_DAYS: usize = 256;
const OLD_FISH_DAYS: usize = 6;
const NEW_FISH_DAYS: usize = 8;

fn simulate_lanternfish_population(buffer: impl BufRead) -> u64 {
    let mut fish_counters: [u64; NEW_FISH_DAYS + 1] = [0; NEW_FISH_DAYS + 1];
    for value in buffer.lines().next().unwrap().unwrap().split(',') {
        let days = u8::from_str(value).unwrap();
        fish_counters[days as usize] += 1;
    }
    for day in 0..MAX_DAYS {
        let zero_day_index = day % fish_counters.len();
        let zero_day_fish = fish_counters[zero_day_index];
        fish_counters[zero_day_index] = 0;
        fish_counters[(OLD_FISH_DAYS + day + 1) % fish_counters.len()] += zero_day_fish;
        fish_counters[(NEW_FISH_DAYS + day + 1) % fish_counters.len()] += zero_day_fish;
    }
    fish_counters.iter().sum()
}

#[test]
fn example_test() {
    let buffer = r#"3,4,3,1,2
"#
    .as_bytes();
    assert_eq!(simulate_lanternfish_population(buffer), 26984457539);
}
