use std::collections::BTreeSet;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", fold_transparent_paper(std::io::stdin().lock()));
}

fn fold_transparent_paper(mut buffer: impl BufRead) -> usize {
    let mut dots = read_dots(&mut buffer);
    for (fold_type, coordinate) in buffer.lines().map(|v| parse_fold(&v.unwrap())).take(1) {
        match fold_type {
            FoldType::X => fold_by_x(coordinate, &mut dots),
            FoldType::Y => fold_by_y(coordinate, &mut dots),
        }
    }
    dots.len()
}

fn fold_by_x(c: u32, dots: &mut BTreeSet<(u32, u32)>) {
    dots.retain(|(x, _)| *x != c);
    let mut folded = Vec::new();
    dots.retain(|(x, y)| {
        if *x > c {
            folded.push((2 * c - *x, *y));
            false
        } else {
            true
        }
    });
    dots.extend(folded.into_iter());
}

fn fold_by_y(c: u32, dots: &mut BTreeSet<(u32, u32)>) {
    dots.retain(|(_, y)| *y != c);
    let mut folded = Vec::new();
    dots.retain(|(x, y)| {
        if *y > c {
            folded.push((*x, 2 * c - *y));
            false
        } else {
            true
        }
    });
    dots.extend(folded.into_iter());
}

fn read_dots(buffer: &mut impl BufRead) -> BTreeSet<(u32, u32)> {
    let mut dots = BTreeSet::new();
    loop {
        let mut line = String::new();
        buffer.read_line(&mut line).unwrap();
        strip_line_break(&mut line);
        if line.is_empty() {
            break;
        }
        let (x, y) = line.split_once(',').unwrap();
        dots.insert((u32::from_str(x).unwrap(), u32::from_str(y).unwrap()));
    }
    dots
}

fn strip_line_break(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
    }
    if line.ends_with('\r') {
        line.pop();
    }
}

enum FoldType {
    X,
    Y,
}

fn parse_fold(line: &str) -> (FoldType, u32) {
    let (name, value) = line.split_once('=').unwrap();
    (
        if name.ends_with('x') {
            FoldType::X
        } else {
            FoldType::Y
        },
        u32::from_str(value).unwrap(),
    )
}

#[test]
fn example_test() {
    let buffer = r#"6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5
"#
    .as_bytes();
    assert_eq!(fold_transparent_paper(buffer), 17);
}
