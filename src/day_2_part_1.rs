use std::io::BufRead;
use std::str::FromStr;

fn main() {
    let position = move_submarine(std::io::stdin().lock());
    println!("{:?}", position);
    println!("{}", product(&position));
}

fn product(value: &Position) -> i64 {
    value.horizontal * value.depth
}

#[derive(Debug, Eq, PartialEq)]
struct Position {
    horizontal: i64,
    depth: i64,
}

fn move_submarine(buffer: impl BufRead) -> Position {
    let mut position = Position {
        horizontal: 0,
        depth: 0,
    };
    for line in buffer.lines() {
        let command_str = line.unwrap();
        let command = parse_command(command_str.as_str());
        match command.direction {
            "forward" => position.horizontal += command.shift,
            "down" => position.depth += command.shift,
            "up" => position.depth -= command.shift,
            _ => (),
        }
    }
    position
}

struct Command<'a> {
    direction: &'a str,
    shift: i64,
}

fn parse_command(value: &str) -> Command {
    let separator = value.find(' ').unwrap();
    Command {
        direction: &value[0..separator],
        shift: i64::from_str(&value[separator + 1..value.len()]).unwrap(),
    }
}

#[test]
fn example_test() {
    let buffer = r#"forward 5
down 5
forward 8
up 3
down 8
forward 2
"#
    .as_bytes();
    let position = move_submarine(buffer);
    assert_eq!(
        position,
        Position {
            horizontal: 15,
            depth: 10
        }
    );
    assert_eq!(product(&position), 150);
}
