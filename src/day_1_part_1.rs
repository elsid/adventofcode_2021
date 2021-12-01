use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", count_increased_height(std::io::stdin().lock()));
}

fn count_increased_height(buffer: impl BufRead) -> usize {
    let mut prev = None;
    let mut increased: usize = 0;
    for line in buffer.lines() {
        let height = i64::from_str(line.unwrap().as_str()).unwrap();
        if let Some(prev_height) = prev {
            increased += (prev_height < height) as usize;
        }
        prev = Some(height);
    }
    increased
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
    assert_eq!(count_increased_height(buffer), 7);
}
