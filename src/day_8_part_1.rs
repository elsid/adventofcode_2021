use std::io::BufRead;

fn main() {
    println!("{}", count_digits(std::io::stdin().lock()));
}

fn count_digits(buffer: impl BufRead) -> usize {
    let mut result = 0;
    for line in buffer.lines() {
        let line_str = line.unwrap();
        let (_, output) = line_str.split_once(" | ").unwrap();
        for word in output.split(' ') {
            match word.len() {
                2 | 4 | 3 | 7 => result += 1,
                _ => (),
            }
        }
    }
    result
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
    assert_eq!(count_digits(buffer), 26);
}
