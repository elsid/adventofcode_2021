use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::BufRead;

fn main() {
    println!("{}", count_cave_paths(std::io::stdin().lock()));
}

fn count_cave_paths(buffer: impl BufRead) -> usize {
    let mut nodes = BTreeMap::<String, Vec<String>>::new();
    let mut add_connection = |src: &str, dst: &str| {
        nodes
            .entry(String::from(src))
            .or_default()
            .push(String::from(dst));
    };
    for line in buffer.lines() {
        let line_str = line.unwrap();
        let (a, b) = line_str.split_once('-').unwrap();
        add_connection(a, b);
        add_connection(b, a);
    }
    let mut paths = vec![Path {
        last: "start",
        visited: ["start"].iter().copied().collect(),
    }];
    let mut incoming = VecDeque::new();
    incoming.push_back(0usize);
    let mut finished_paths = 0;
    while let Some(path_index) = incoming.pop_front() {
        if paths[path_index].last == "end" {
            finished_paths += 1;
            continue;
        }
        for neighbour in nodes[paths[path_index].last].iter() {
            let small_cave = is_small_cave(neighbour);
            if small_cave && paths[path_index].visited.contains(neighbour.as_str()) {
                continue;
            }
            let mut neighbour_path = paths[path_index].clone();
            neighbour_path.last = neighbour;
            if small_cave {
                neighbour_path.visited.insert(neighbour);
            }
            paths.push(neighbour_path);
            incoming.push_back(paths.len() - 1);
        }
    }
    finished_paths
}

#[derive(Clone)]
struct Path<'a> {
    last: &'a str,
    visited: BTreeSet<&'a str>,
}

fn is_small_cave(name: &str) -> bool {
    name.chars().all(|v| v.is_ascii_lowercase())
}

#[test]
fn small_example_test() {
    let buffer = r#"start-A
start-b
A-c
A-b
b-d
A-end
b-end
"#
    .as_bytes();
    assert_eq!(count_cave_paths(buffer), 10);
}

#[test]
fn example_test() {
    let buffer = r#"dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc
"#
    .as_bytes();
    assert_eq!(count_cave_paths(buffer), 19);
}

#[test]
fn large_example_test() {
    let buffer = r#"fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW
"#
    .as_bytes();
    assert_eq!(count_cave_paths(buffer), 226);
}
