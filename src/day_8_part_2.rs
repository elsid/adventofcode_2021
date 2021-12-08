use std::collections::BTreeMap;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", sum_decoded_numbers(std::io::stdin().lock()));
}

fn sum_decoded_numbers(buffer: impl BufRead) -> u64 {
    buffer.lines().map(|v| decode_line(&v.unwrap())).sum()
}

fn decode_line(line: &str) -> u64 {
    let (signal, output) = line.split_once(" | ").unwrap();
    let mut signals = BTreeMap::new();
    let mut digits = [None; 10];
    for word in signal.split(' ') {
        if let Some(digit) = get_digit_by_signal_len(word.len()) {
            signals.insert(make_word_key(word), digit);
            digits[digit as usize] = Some(word.as_bytes());
        }
    }
    for word in signal.split(' ') {
        match word.len() {
            5 => {
                let digit = if contains_bytes(word.as_bytes(), &digits[1].unwrap()) {
                    3
                } else if contains_at_least_n_bytes(word.as_bytes(), digits[4].unwrap(), 3) {
                    5
                } else {
                    2
                };
                signals.insert(make_word_key(word), digit);
            }
            6 => {
                let digit = if contains_bytes(word.as_bytes(), digits[4].unwrap()) {
                    9
                } else if contains_bytes(word.as_bytes(), digits[1].unwrap()) {
                    0
                } else {
                    6
                };
                signals.insert(make_word_key(word), digit);
            }
            _ => (),
        }
    }
    u64::from_str(
        &output
            .split(' ')
            .map(|word| (signals[&make_word_key(word)] + b'0') as char)
            .collect::<String>(),
    )
    .unwrap()
}

fn get_digit_by_signal_len(value: usize) -> Option<u8> {
    match value {
        2 => Some(1),
        3 => Some(7),
        4 => Some(4),
        7 => Some(8),
        _ => None,
    }
}

fn make_word_key(word: &str) -> Vec<u8> {
    let mut key: Vec<u8> = word.as_bytes().iter().cloned().collect();
    key.sort();
    key
}

fn contains_bytes(word: &[u8], bytes: &[u8]) -> bool {
    bytes.iter().all(|w| word.iter().any(|v| *v == *w))
}

fn contains_at_least_n_bytes(word: &[u8], bytes: &[u8], n: usize) -> bool {
    bytes
        .iter()
        .filter(|w| word.iter().any(|v| *v == **w))
        .count()
        >= n
}

#[test]
fn example_test() {
    let buffer =
        r#"be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga | gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf | gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf | cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd | ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg | gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc | fgae cfgab fg bagce
"#
        .as_bytes();
    assert_eq!(sum_decoded_numbers(buffer), 61229);
}
