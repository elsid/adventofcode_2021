use std::io::BufRead;

fn main() {
    println!(
        "{}",
        calculate_total_syntax_error_score(std::io::stdin().lock())
    );
}

fn calculate_total_syntax_error_score(buffer: impl BufRead) -> u64 {
    buffer
        .lines()
        .map(|v| get_syntax_error_score(&v.unwrap()))
        .sum()
}

fn get_syntax_error_score(line: &str) -> u64 {
    let mut open = Vec::new();
    for symbol in line.chars() {
        match symbol {
            '(' | '[' | '{' | '<' => open.push(symbol),
            ')' | ']' | '}' | '>' => {
                if let Some(pair) = open.pop() {
                    if get_matching_close(pair) != symbol {
                        return get_symbol_score(symbol);
                    }
                }
            }
            _ => (),
        }
    }
    0
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

fn get_symbol_score(v: char) -> u64 {
    match v {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
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
    assert_eq!(calculate_total_syntax_error_score(buffer), 26397);
}
