use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!(
        "{:?}",
        find_first_bingo_winner_score(std::io::stdin().lock())
    );
}

const ROW_SIZE: usize = 5;
const COLUMN_SIZE: usize = ROW_SIZE;
const BOARD_SIZE: usize = ROW_SIZE * COLUMN_SIZE;

#[derive(Clone)]
struct Board {
    numbers: [u8; BOARD_SIZE],
    column_counters: [u8; COLUMN_SIZE],
    row_counters: [u8; ROW_SIZE],
    marked: [bool; BOARD_SIZE],
}

fn find_first_bingo_winner_score(mut buffer: impl BufRead) -> Option<u64> {
    let numbers = read_numbers(&mut buffer);
    let mut boards = read_boards(buffer);
    for number in numbers.iter() {
        for board in boards.iter_mut() {
            mark_number(*number, board);
            if is_winner(board) {
                return Some(*number as u64 * get_unmarked_sum(board));
            }
        }
    }
    None
}

fn mark_number(number: u8, board: &mut Board) {
    for i in 0..BOARD_SIZE {
        if board.marked[i] || board.numbers[i] != number {
            continue;
        }
        board.marked[i] = true;
        board.column_counters[get_column_index(i)] += 1;
        board.row_counters[get_row_index(i)] += 1;
    }
}

fn is_winner(board: &Board) -> bool {
    board
        .column_counters
        .iter()
        .any(|v| *v as usize == COLUMN_SIZE)
        || board.row_counters.iter().any(|v| *v as usize == ROW_SIZE)
}

fn get_unmarked_sum(board: &Board) -> u64 {
    (0..BOARD_SIZE)
        .into_iter()
        .filter(|i| !board.marked[*i])
        .map(|i| board.numbers[i] as u64)
        .sum()
}

fn get_row_index(index: usize) -> usize {
    index / COLUMN_SIZE
}

fn get_column_index(index: usize) -> usize {
    index % COLUMN_SIZE
}

fn read_numbers(buffer: &mut impl BufRead) -> Vec<u8> {
    let mut line = String::new();
    buffer.read_line(&mut line).unwrap();
    strip_line_break(&mut line);
    line.split(',')
        .by_ref()
        .map(|v| u8::from_str(v).unwrap())
        .collect()
}

fn strip_line_break(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
    }
    if line.ends_with('\r') {
        line.pop();
    }
}

fn read_boards(buffer: impl BufRead) -> Vec<Board> {
    let mut result = Vec::new();
    let mut board = Board {
        numbers: [0; BOARD_SIZE],
        column_counters: [0; 5],
        row_counters: [0; 5],
        marked: [false; BOARD_SIZE],
    };
    let mut index = 0;
    for line in buffer.lines() {
        let row = line.unwrap();
        if row.is_empty() {
            if index > 0 {
                result.push(board.clone());
                board.numbers.fill(0);
                index = 0;
            }
            continue;
        }
        row.split(' ').filter(|v| !v.is_empty()).for_each(|v| {
            board.numbers[index] = u8::from_str(v).unwrap();
            index += 1;
        });
    }
    result.push(board.clone());
    result
}

#[test]
fn example_test() {
    let buffer = r#"7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7
"#
    .as_bytes();
    assert_eq!(find_first_bingo_winner_score(buffer), Some(4512));
}
