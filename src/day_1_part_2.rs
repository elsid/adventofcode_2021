use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!(
        "{}",
        count_increased_height_with_window(std::io::stdin().lock())
    );
}

fn count_increased_height_with_window(buffer: impl BufRead) -> usize {
    let mut prev_window_sum = None;
    let mut window: [i64; 3] = [0; 3];
    let mut window_sum: i64 = 0;
    let mut count: usize = 0;
    let mut increased: usize = 0;
    for line in buffer.lines() {
        let height = i64::from_str(line.unwrap().as_str()).unwrap();
        add_to_window(height, &mut window, &mut window_sum);
        count += 1;
        if count < 3 {
            continue;
        }
        if let Some(prev) = prev_window_sum {
            increased += (prev < window_sum) as usize;
        }
        prev_window_sum = Some(window_sum);
    }
    increased
}

fn add_to_window(value: i64, window: &mut [i64], sum: &mut i64) {
    *sum -= window[0];
    for i in 1..window.len() {
        window[i - 1] = window[i];
    }
    window[window.len() - 1] = value;
    *sum += value;
}

#[test]
fn example_test() {
    let buffer: &[u8] = r#"199
200
208
210
200
207
240
269
260
263
"#
    .as_bytes();
    assert_eq!(count_increased_height_with_window(buffer), 5);
}
