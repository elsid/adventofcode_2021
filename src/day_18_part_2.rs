use std::io::BufRead;

fn main() {
    println!(
        "{}",
        find_max_sum_snailfish_magnitude(std::io::stdin().lock())
    );
}

#[derive(Debug, Eq, PartialEq)]
struct Tree {
    root: usize,
    first_regular: usize,
    last_regular: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Node {
    parent: Option<(usize, Side)>,
    value: Value,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Value {
    Pair {
        left: usize,
        right: usize,
    },
    Regular {
        number: u8,
        prev: Option<usize>,
        next: Option<usize>,
    },
}

fn find_max_sum_snailfish_magnitude(buffer: impl BufRead) -> u64 {
    let mut nodes = Vec::new();
    let trees: Vec<Tree> = buffer
        .lines()
        .map(|v| parse_tree(&v.unwrap(), &mut nodes))
        .collect();
    let mut max_magnitude = 0;
    for i in 0..trees.len() - 1 {
        let left = &trees[i];
        for right in &trees[i + 1..trees.len()] {
            {
                let mut nodes_copy = nodes.clone();
                let magnitude =
                    calculate_magnitude(sum_trees(left, right, &mut nodes_copy).root, &nodes_copy);
                if max_magnitude < magnitude {
                    max_magnitude = magnitude;
                }
            }
            {
                let mut nodes_copy = nodes.clone();
                let magnitude =
                    calculate_magnitude(sum_trees(right, left, &mut nodes_copy).root, &nodes_copy);
                if max_magnitude < magnitude {
                    max_magnitude = magnitude;
                }
            }
        }
    }
    max_magnitude
}

fn calculate_magnitude(node: usize, nodes: &[Node]) -> u64 {
    match nodes[node].value {
        Value::Pair { left, right } => {
            3 * calculate_magnitude(left, nodes) + 2 * calculate_magnitude(right, nodes)
        }
        Value::Regular { number, .. } => number as u64,
    }
}

fn sum_trees(left: &Tree, right: &Tree, nodes: &mut Vec<Node>) -> Tree {
    let mut result = combine_trees(left, right, nodes);
    reduce_tree(&mut result, nodes);
    result
}

fn combine_trees(left: &Tree, right: &Tree, nodes: &mut Vec<Node>) -> Tree {
    let root = nodes.len();
    nodes.push(Node {
        parent: None,
        value: Value::Pair {
            left: left.root,
            right: right.root,
        },
    });
    nodes[left.root].parent = Some((root, Side::Left));
    nodes[right.root].parent = Some((root, Side::Right));
    match &mut nodes[left.last_regular].value {
        Value::Regular { next, .. } => *next = Some(right.first_regular),
        _ => panic!("{} {:?}", left.last_regular, nodes[left.last_regular]),
    }
    match &mut nodes[right.first_regular].value {
        Value::Regular { prev, .. } => *prev = Some(left.last_regular),
        _ => panic!("{} {:?}", right.first_regular, nodes[right.first_regular]),
    }
    Tree {
        root,
        first_regular: left.first_regular,
        last_regular: right.last_regular,
    }
}

fn reduce_tree(tree: &mut Tree, nodes: &mut Vec<Node>) {
    loop {
        if let Some(node) = find_explodable_node(tree.root, nodes) {
            let (left, right) = match &nodes[node].value {
                Value::Pair { left, right } => (*left, *right),
                _ => panic!("{} {:?}", node, nodes[node]),
            };
            explode_node(node, nodes);
            if left == tree.first_regular {
                tree.first_regular = node;
            } else if right == tree.last_regular {
                tree.last_regular = node;
            }
            continue;
        }
        if let Some(node) = find_splittable_node(tree.root, nodes) {
            split_node(node, nodes);
            if node == tree.first_regular {
                match &nodes[node].value {
                    Value::Pair { left, .. } => tree.first_regular = *left,
                    _ => panic!("{} {:?}", node, nodes[node]),
                }
            } else if node == tree.last_regular {
                match &nodes[node].value {
                    Value::Pair { right, .. } => tree.last_regular = *right,
                    _ => panic!("{} {:?}", node, nodes[node]),
                }
            }
            continue;
        }
        break;
    }
}

fn explode_node(pair_index: usize, nodes: &mut Vec<Node>) {
    let prev_regular;
    let next_regular;
    match nodes[pair_index].value.clone() {
        Value::Pair { left, right } => {
            match nodes[left].value.clone() {
                Value::Regular { number, prev, .. } => {
                    prev_regular = prev;
                    if let Some(i) = prev {
                        match &mut nodes[i].value {
                            Value::Regular {
                                number: prev_number,
                                next,
                                ..
                            } => {
                                *prev_number += number;
                                *next = Some(pair_index);
                            }
                            _ => panic!("{} {:?}", i, nodes[i]),
                        }
                    }
                }
                _ => panic!("{} {:?}", left, nodes[left]),
            }
            nodes[left].parent = None;
            match nodes[right].value.clone() {
                Value::Regular { number, next, .. } => {
                    next_regular = next;
                    if let Some(i) = next {
                        match &mut nodes[i].value {
                            Value::Regular {
                                number: next_number,
                                prev,
                                ..
                            } => {
                                *next_number += number;
                                *prev = Some(pair_index);
                            }
                            _ => panic!("{} {:?}", i, nodes[i]),
                        }
                    }
                }
                _ => panic!("{} {:?}", right, nodes[right]),
            }
            nodes[right].parent = None;
        }
        _ => panic!("{} {:?}", pair_index, nodes[pair_index]),
    }
    nodes[pair_index].value = Value::Regular {
        number: 0,
        prev: prev_regular,
        next: next_regular,
    };
}

fn split_node(regular_index: usize, nodes: &mut Vec<Node>) {
    let (number, prev, next) = match &nodes[regular_index].value {
        Value::Regular { number, prev, next } => (*number, *prev, *next),
        _ => panic!("{} {:?}", regular_index, nodes[regular_index]),
    };
    let left = nodes.len();
    let right = nodes.len() + 1;
    nodes.push(Node {
        parent: Some((regular_index, Side::Left)),
        value: Value::Regular {
            number: number / 2,
            prev,
            next: Some(right),
        },
    });
    nodes.push(Node {
        parent: Some((regular_index, Side::Right)),
        value: Value::Regular {
            number: number / 2 + number % 2,
            prev: Some(left),
            next,
        },
    });
    if let Some(i) = prev {
        match &mut nodes[i].value {
            Value::Regular { next, .. } => *next = Some(left),
            _ => panic!("{} {:?}", i, nodes[i]),
        }
    }
    if let Some(i) = next {
        match &mut nodes[i].value {
            Value::Regular { prev, .. } => *prev = Some(right),
            _ => panic!("{} {:?}", i, nodes[i]),
        }
    }
    nodes[regular_index].value = Value::Pair { left, right };
}

fn find_explodable_node(node: usize, nodes: &[Node]) -> Option<usize> {
    find_explodable_node_recursive(node, 0, nodes)
}

fn find_explodable_node_recursive(node: usize, depth: usize, nodes: &[Node]) -> Option<usize> {
    if let Value::Pair { left, right } = nodes[node].value {
        if depth >= 4
            && matches!(nodes[left].value, Value::Regular { .. })
            && matches!(nodes[right].value, Value::Regular { .. })
        {
            return Some(node);
        }
        match find_explodable_node_recursive(left, depth + 1, nodes) {
            None => (),
            v => return v,
        }
        match find_explodable_node_recursive(right, depth + 1, nodes) {
            None => (),
            v => return v,
        }
    }
    None
}

fn find_splittable_node(node: usize, nodes: &[Node]) -> Option<usize> {
    match nodes[node].value {
        Value::Pair { left, right } => {
            match find_splittable_node(left, nodes) {
                None => (),
                v => return v,
            }
            match find_splittable_node(right, nodes) {
                None => (),
                v => return v,
            }
        }
        Value::Regular { number, .. } => {
            if number >= 10 {
                return Some(node);
            }
        }
    }
    None
}

fn parse_tree(line: &str, nodes: &mut Vec<Node>) -> Tree {
    let mut states: Vec<(usize, Side)> = Vec::new();
    let root = nodes.len();
    let mut prev = None;
    let mut first_regular = usize::MAX;
    for symbol in line.bytes() {
        match symbol {
            b'[' => {
                let index = nodes.len();
                update_parent(index, &states, nodes);
                nodes.push(Node {
                    parent: states.last().cloned(),
                    value: Value::Pair {
                        left: usize::MAX,
                        right: usize::MAX,
                    },
                });
                states.push((index, Side::Left));
            }
            b']' => {
                states.pop();
            }
            b',' => {
                states.last_mut().unwrap().1 = Side::Right;
            }
            v => {
                let index = nodes.len();
                update_parent(index, &states, nodes);
                nodes.push(Node {
                    parent: states.last().cloned(),
                    value: Value::Regular {
                        number: v - b'0',
                        prev,
                        next: None,
                    },
                });
                if let Some(prev_index) = prev {
                    match &mut nodes[prev_index].value {
                        Value::Regular { next, .. } => *next = Some(index),
                        _ => panic!("{:?}", nodes[prev_index]),
                    }
                } else {
                    first_regular = index;
                }
                prev = Some(index);
            }
        }
    }
    Tree {
        root,
        first_regular,
        last_regular: prev.unwrap(),
    }
}

fn update_parent(index: usize, states: &[(usize, Side)], nodes: &mut Vec<Node>) {
    if let Some((parent, side)) = states.last().cloned() {
        match &mut nodes[parent].value {
            Value::Pair { left, right } => match side {
                Side::Left => *left = index,
                Side::Right => *right = index,
            },
            _ => panic!("{:?}", nodes[parent]),
        }
    }
}

#[cfg(test)]
fn node_to_string(node: usize, nodes: &[Node]) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    write_node(node, nodes, &mut buffer);
    String::from_utf8(buffer).unwrap()
}

#[cfg(test)]
fn write_node(node: usize, nodes: &[Node], buffer: &mut impl std::io::Write) {
    match nodes[node].value {
        Value::Pair { left, right } => {
            buffer.write_all(&[b'[']).unwrap();
            write_node(left, nodes, buffer);
            buffer.write_all(&[b',']).unwrap();
            write_node(right, nodes, buffer);
            buffer.write_all(&[b']']).unwrap();
        }
        Value::Regular { number, .. } => {
            buffer.write_all(format!("{}", number).as_bytes()).unwrap();
        }
    }
}

#[test]
fn parse_tree_test() {
    let mut nodes = Vec::new();
    assert_eq!(
        parse_tree("[1,2]", &mut nodes),
        Tree {
            root: 0,
            first_regular: 1,
            last_regular: 2
        }
    );
    assert_eq!(
        nodes,
        vec![
            Node {
                parent: None,
                value: Value::Pair { left: 1, right: 2 }
            },
            Node {
                parent: Some((0, Side::Left)),
                value: Value::Regular {
                    number: 1,
                    prev: None,
                    next: Some(2)
                }
            },
            Node {
                parent: Some((0, Side::Right)),
                value: Value::Regular {
                    number: 2,
                    prev: Some(1),
                    next: None
                }
            },
        ]
    );
    assert_eq!(
        parse_tree("[[1,2],3]", &mut nodes),
        Tree {
            root: 3,
            first_regular: 5,
            last_regular: 7
        }
    );
    assert_eq!(
        nodes,
        vec![
            Node {
                parent: None,
                value: Value::Pair { left: 1, right: 2 }
            },
            Node {
                parent: Some((0, Side::Left)),
                value: Value::Regular {
                    number: 1,
                    prev: None,
                    next: Some(2)
                }
            },
            Node {
                parent: Some((0, Side::Right)),
                value: Value::Regular {
                    number: 2,
                    prev: Some(1),
                    next: None
                }
            },
            Node {
                parent: None,
                value: Value::Pair { left: 4, right: 7 }
            },
            Node {
                parent: Some((3, Side::Left)),
                value: Value::Pair { left: 5, right: 6 }
            },
            Node {
                parent: Some((4, Side::Left)),
                value: Value::Regular {
                    number: 1,
                    prev: None,
                    next: Some(6)
                }
            },
            Node {
                parent: Some((4, Side::Right)),
                value: Value::Regular {
                    number: 2,
                    prev: Some(5),
                    next: Some(7)
                }
            },
            Node {
                parent: Some((3, Side::Right)),
                value: Value::Regular {
                    number: 3,
                    prev: Some(6),
                    next: None
                }
            },
        ]
    )
}

#[test]
fn node_to_string_test() {
    let mut nodes = Vec::new();
    let tree = parse_tree("[[1,2],3]", &mut nodes);
    assert_eq!(node_to_string(tree.root, &nodes), "[[1,2],3]");
}

#[test]
fn calculate_magnitude_1_test() {
    let mut nodes = Vec::new();
    let tree = parse_tree("[[1,2],[[3,4],5]]", &mut nodes);
    assert_eq!(calculate_magnitude(tree.root, &nodes), 143);
}

#[test]
fn combine_trees_test() {
    use Side::*;
    use Value::*;
    let mut nodes = Vec::new();
    let left = parse_tree("[1,2]", &mut nodes);
    let right = parse_tree("[[3,4],5]", &mut nodes);
    let tree = combine_trees(&left, &right, &mut nodes);
    assert_eq!(node_to_string(tree.root, &nodes), "[[1,2],[[3,4],5]]");
    assert_eq!(
        nodes,
        vec![
            Node {
                parent: Some((8, Side::Left)),
                value: Pair { left: 1, right: 2 }
            },
            Node {
                parent: Some((0, Left)),
                value: Regular {
                    number: 1,
                    prev: None,
                    next: Some(2)
                }
            },
            Node {
                parent: Some((0, Right)),
                value: Regular {
                    number: 2,
                    prev: Some(1),
                    next: Some(5)
                }
            },
            Node {
                parent: Some((8, Side::Right)),
                value: Pair { left: 4, right: 7 }
            },
            Node {
                parent: Some((3, Left)),
                value: Pair { left: 5, right: 6 }
            },
            Node {
                parent: Some((4, Left)),
                value: Regular {
                    number: 3,
                    prev: Some(2),
                    next: Some(6)
                }
            },
            Node {
                parent: Some((4, Right)),
                value: Regular {
                    number: 4,
                    prev: Some(5),
                    next: Some(7)
                }
            },
            Node {
                parent: Some((3, Right)),
                value: Regular {
                    number: 5,
                    prev: Some(6),
                    next: None
                }
            },
            Node {
                parent: None,
                value: Pair { left: 0, right: 3 }
            }
        ]
    );
}

#[test]
fn find_explodable_node_test() {
    let mut nodes = Vec::new();
    let tree = parse_tree("[[[[[9,8],1],2],3],4]", &mut nodes);
    assert_eq!(find_explodable_node(tree.root, &nodes), Some(4));
}

#[test]
fn find_splittable_node_test() {
    let mut nodes = Vec::new();
    let tree = parse_tree("[:,0]", &mut nodes);
    assert_eq!(find_splittable_node(tree.root, &nodes), Some(1));
}

#[test]
fn explode_node_test() {
    let cases = [
        ("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]"),
        ("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]"),
        ("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]"),
        (
            "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]",
            "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
        ),
        (
            "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
            "[[3,[2,[8,0]]],[9,[5,[7,0]]]]",
        ),
    ];
    for (input, expected) in cases {
        let mut nodes = Vec::new();
        let tree = parse_tree(input, &mut nodes);
        let node = find_explodable_node(tree.root, &nodes);
        assert!(matches!(node, Some(..)), "input: {}", input);
        explode_node(node.unwrap(), &mut nodes);
        assert_eq!(
            node_to_string(tree.root, &nodes),
            expected,
            "input: {}",
            input
        );
    }
}

#[test]
fn split_node_test() {
    let cases = [
        ("[:,0]", "[[5,5],0]"),
        ("[;,1]", "[[5,6],1]"),
        ("[<,2]", "[[6,6],2]"),
    ];
    for (input, expected) in cases {
        let mut nodes = Vec::new();
        let tree = parse_tree(input, &mut nodes);
        let node = find_splittable_node(tree.root, &nodes);
        assert!(matches!(node, Some(..)), "input: {}", input);
        split_node(node.unwrap(), &mut nodes);
        assert_eq!(
            node_to_string(tree.root, &nodes),
            expected,
            "input: {}",
            input
        );
    }
}

#[test]
fn calculate_magnitude_2_test() {
    let mut nodes = Vec::new();
    let tree = parse_tree("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", &mut nodes);
    assert_eq!(calculate_magnitude(tree.root, &nodes), 1384);
}

#[test]
fn example_test() {
    let buffer = r#"[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
"#
    .as_bytes();
    assert_eq!(find_max_sum_snailfish_magnitude(buffer), 3993);
}
