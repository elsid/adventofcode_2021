use std::io::BufRead;

fn main() {
    println!("{}", calculate_life_support_rating(std::io::stdin().lock()));
}

fn calculate_life_support_rating(buffer: impl BufRead) -> u64 {
    let mut values = Vec::new();
    for line in buffer.lines() {
        values.push(Vec::from(line.unwrap().as_bytes()));
    }
    oxygen_generator_rating(values.clone()) * co2_scrubber_rating(values)
}

fn oxygen_generator_rating(values: Vec<Vec<u8>>) -> u64 {
    filter_values(&|zero_count, one_count| one_count >= zero_count, values)
}

fn co2_scrubber_rating(values: Vec<Vec<u8>>) -> u64 {
    filter_values(&|zero_count, one_count| one_count < zero_count, values)
}

fn filter_values(get_bit: &impl Fn(usize, usize) -> bool, mut values: Vec<Vec<u8>>) -> u64 {
    let mut index = 0;
    while values.len() > 1 {
        let (zero_count, one_count) = count_bits_at(&values, index);
        let bit = if get_bit(zero_count, one_count) {
            b'1'
        } else {
            b'0'
        };
        values.retain(|value| value[index] == bit);
        index += 1;
    }
    if values.is_empty() {
        return 0;
    }
    u64::from_str_radix(String::from_utf8_lossy(&values[0]).as_ref(), 2).unwrap()
}

fn count_bits_at(values: &[Vec<u8>], index: usize) -> (usize, usize) {
    let mut zero_count = 0;
    let mut one_count = 0;
    for value in values.iter() {
        match value[index] {
            b'0' => zero_count += 1,
            b'1' => one_count += 1,
            _ => (),
        }
    }
    (zero_count, one_count)
}

#[test]
fn example_test() {
    let buffer = r#"00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010
"#
    .as_bytes();
    assert_eq!(calculate_life_support_rating(buffer), 230);
}
