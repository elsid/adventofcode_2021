use std::io::BufRead;
use std::str::FromStr;

fn main() {
    let state = move_submarine_with_aim(std::io::stdin().lock());
    println!("{:?}", state);
    println!("{}", position_product(&state));
}

fn position_product(value: &State) -> i64 {
    value.horizontal * value.depth
}

#[derive(Debug, Eq, PartialEq)]
struct State {
    horizontal: i64,
    depth: i64,
    aim: i64,
}

fn move_submarine_with_aim(buffer: impl BufRead) -> State {
    let mut state = State {
        horizontal: 0,
        depth: 0,
        aim: 0,
    };
    for line in buffer.lines() {
        let command_str = line.unwrap();
        let command = parse_command(command_str.as_str());
        match command.direction {
            "forward" => {
                state.horizontal += command.shift;
                state.depth += state.aim * command.shift;
            }
            "down" => state.aim += command.shift,
            "up" => state.aim -= command.shift,
            _ => (),
        }
    }
    state
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
    let position = move_submarine_with_aim(buffer);
    assert_eq!(
        position,
        State {
            horizontal: 15,
            depth: 60,
            aim: 10,
        }
    );
    assert_eq!(position_product(&position), 900);
}
