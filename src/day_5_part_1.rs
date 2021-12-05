use std::collections::BTreeMap;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", count_points_of_intersection(std::io::stdin().lock()));
}

fn count_points_of_intersection(buffer: impl BufRead) -> usize {
    let mut area: BTreeMap<Point, usize> = BTreeMap::new();
    for line in buffer.lines() {
        fill_segment(&parse_segment(&line.unwrap()), &mut area);
    }
    area.values().filter(|v| **v >= 2).count()
}

fn fill_segment(segment: &Segment, area: &mut BTreeMap<Point, usize>) {
    if segment.begin.x == segment.end.x {
        let begin = segment.begin.y.min(segment.end.y);
        let end = segment.begin.y.max(segment.end.y);
        for y in begin..=end {
            area.entry(Point {
                x: segment.begin.x,
                y,
            })
            .and_modify(|v| *v += 1)
            .or_insert(1);
        }
        return;
    }
    if segment.begin.y == segment.end.y {
        let begin = segment.begin.x.min(segment.end.x);
        let end = segment.begin.x.max(segment.end.x);
        for x in begin..=end {
            area.entry(Point {
                x,
                y: segment.begin.y,
            })
            .and_modify(|v| *v += 1)
            .or_insert(1);
        }
        return;
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Point {
    x: i64,
    y: i64,
}

struct Segment {
    begin: Point,
    end: Point,
}

fn parse_segment(line: &str) -> Segment {
    let mut segment: [[i64; 2]; 2] = [[0; 2], [0; 2]];
    for (point_str, point) in line.split(" -> ").zip(segment.iter_mut()) {
        for (str, coordinate) in point_str.split(',').zip(point.iter_mut()) {
            *coordinate = i64::from_str(str).unwrap();
        }
    }
    Segment {
        begin: Point {
            x: segment[0][0],
            y: segment[0][1],
        },
        end: Point {
            x: segment[1][0],
            y: segment[1][1],
        },
    }
}

#[test]
fn example_test() {
    let buffer = r#"0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2
"#
    .as_bytes();
    assert_eq!(count_points_of_intersection(buffer), 5);
}
