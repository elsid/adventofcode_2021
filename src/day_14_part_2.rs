use std::collections::BTreeMap;
use std::io::BufRead;

fn main() {
    println!("{}", build_polymer(std::io::stdin().lock()));
}

fn build_polymer(mut buffer: impl BufRead) -> u64 {
    let polymer = read_polymer(&mut buffer);
    buffer.read_line(&mut String::new()).unwrap();
    let rules: BTreeMap<(u8, u8), u8> = buffer
        .lines()
        .map(|v| parse_pair_insertion_rule(&v.unwrap()))
        .collect();
    let mut pairs: BTreeMap<(u8, u8), u64> = BTreeMap::new();
    for i in 1..polymer.len() {
        *pairs.entry((polymer[i - 1], polymer[i])).or_default() += 1;
    }
    let mut elements: BTreeMap<u8, u64> = BTreeMap::new();
    for element in polymer.iter() {
        *elements.entry(*element).or_default() += 1;
    }
    for _ in 0..40 {
        let mut new_pairs: BTreeMap<(u8, u8), u64> = BTreeMap::new();
        for (pair, count) in pairs.iter() {
            if let Some(insert) = rules.get(pair) {
                *new_pairs.entry((pair.0, *insert)).or_default() += *count;
                *new_pairs.entry((*insert, pair.1)).or_default() += *count;
                *elements.entry(*insert).or_default() += *count;
            } else {
                *new_pairs.entry(*pair).or_default() += *count;
            }
        }
        pairs = new_pairs;
    }
    elements.values().max().unwrap() - elements.values().min().unwrap()
}

fn read_polymer(buffer: &mut impl BufRead) -> Vec<u8> {
    let mut line = String::new();
    buffer.read_line(&mut line).unwrap();
    strip_line_break(&mut line);
    return line.as_bytes().to_vec();
}

fn strip_line_break(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
    }
    if line.ends_with('\r') {
        line.pop();
    }
}

fn parse_pair_insertion_rule(line: &str) -> ((u8, u8), u8) {
    let (pair, insert) = line.split_once(" -> ").unwrap();
    (
        (pair.as_bytes()[0], pair.as_bytes()[1]),
        insert.as_bytes()[0],
    )
}

#[test]
fn example_test() {
    let buffer = r#"NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C
"#
    .as_bytes();
    assert_eq!(build_polymer(buffer), 2188189693529);
}
