use std::io::BufRead;

fn main() {
    println!(
        "{}",
        calculate_total_completion_score(std::io::stdin().lock())
    );
}

fn calculate_total_completion_score(buffer: impl BufRead) -> u64 {
    let mut scores: Vec<u64> = buffer
        .lines()
        .map(|v| get_completion_score(&v.unwrap()))
        .filter(|v| *v != 0)
        .collect();
    scores.sort();
    scores[scores.len() / 2]
}

fn get_completion_score(line: &str) -> u64 {
    let mut open = Vec::new();
    for symbol in line.chars() {
        match symbol {
            '(' | '[' | '{' | '<' => open.push(symbol),
            ')' | ']' | '}' | '>' => {
                if let Some(pair) = open.last().cloned() {
                    if get_matching_close(pair) != symbol {
                        return 0;
                    }
                    open.pop();
                }
            }
            _ => (),
        }
    }
    open.iter()
        .rev()
        .map(|v| get_symbol_completion_score(*v))
        .fold(0, |total, score| total * 5 + score)
}

fn get_matching_close(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        v => v,
    }
}

fn get_symbol_completion_score(v: char) -> u64 {
    match v {
        '(' => 1,
        '[' => 2,
        '{' => 3,
        '<' => 4,
        _ => 0,
    }
}

#[test]
fn example_test() {
    let buffer = r#"[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]
"#
    .as_bytes();
    assert_eq!(calculate_total_completion_score(buffer), 288957);
}
