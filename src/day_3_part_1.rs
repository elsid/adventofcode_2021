use std::io::BufRead;

fn main() {
    println!("{}", calculate_power_consumption(std::io::stdin().lock()));
}

fn calculate_power_consumption(buffer: impl BufRead) -> u64 {
    let mut zero_bit_counters: [usize; 64] = [0; 64];
    let mut one_bit_counters: [usize; 64] = [0; 64];
    let mut length = 0;
    for line in buffer.lines() {
        let measurement = line.unwrap();
        length = length.max(measurement.len());
        for (index, bit) in measurement.char_indices() {
            match bit {
                '0' => zero_bit_counters[index] += 1,
                '1' => one_bit_counters[index] += 1,
                _ => (),
            }
        }
    }
    get_gamma_rate(&zero_bit_counters[0..length], &one_bit_counters[0..length])
        * get_epsilon_rate(&zero_bit_counters[0..length], &one_bit_counters[0..length])
}

fn get_gamma_rate(zero_bit_counters: &[usize], one_bit_counters: &[usize]) -> u64 {
    fill_bits(
        zero_bit_counters,
        one_bit_counters,
        &|zero_count, one_count| one_count > zero_count,
    )
}

fn get_epsilon_rate(zero_bit_counters: &[usize], one_bit_counters: &[usize]) -> u64 {
    fill_bits(
        zero_bit_counters,
        one_bit_counters,
        &|zero_count, one_count| one_count < zero_count,
    )
}

fn fill_bits(
    zero_bit_counters: &[usize],
    one_bit_counters: &[usize],
    get: &impl Fn(usize, usize) -> bool,
) -> u64 {
    let mut result: u64 = 0;
    let length = zero_bit_counters.len().min(one_bit_counters.len());
    for (index, (zero_count, one_count)) in zero_bit_counters
        .iter()
        .zip(one_bit_counters.iter())
        .enumerate()
    {
        if get(*zero_count, *one_count) {
            result |= 1 << (length - index - 1);
        }
    }
    result
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
    assert_eq!(calculate_power_consumption(buffer), 198);
}
