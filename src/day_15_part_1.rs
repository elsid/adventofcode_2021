use std::collections::BinaryHeap;
use std::io::BufRead;

fn main() {
    println!("{}", calculate_lowest_total_risk(std::io::stdin().lock()));
}

fn calculate_lowest_total_risk(buffer: impl BufRead) -> i64 {
    let grid = parse_grid(buffer);
    type Visited = Vec<bool>;
    type Nodes = BinaryHeap<(i64, usize)>;
    let try_push = |risk, x, y, visited: &Visited, nodes: &mut Nodes| {
        let index = grid.index(x, y);
        if visited[index] {
            return;
        }
        nodes.push(((risk - grid.values[index] as i64), index));
    };
    let mut visited: Visited = std::iter::repeat(false).take(grid.values.len()).collect();
    let mut nodes = Nodes::new();
    nodes.push((0, 0));
    while let Some((node_risk, node_index)) = nodes.pop() {
        visited[node_index] = true;
        if node_index == grid.values.len() - 1 {
            return -node_risk;
        }
        let (node_x, node_y) = grid.position(node_index);
        if node_x > 0 {
            try_push(node_risk, node_x - 1, node_y, &visited, &mut nodes);
        }
        if node_y > 0 {
            try_push(node_risk, node_x, node_y - 1, &visited, &mut nodes);
        }
        if node_x + 1 < grid.width {
            try_push(node_risk, node_x + 1, node_y, &visited, &mut nodes);
        }
        if node_y + 1 < grid.height {
            try_push(node_risk, node_x, node_y + 1, &visited, &mut nodes);
        }
    }
    0
}

struct Grid {
    values: Vec<u8>,
    width: usize,
    height: usize,
}

impl Grid {
    fn position(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }
}

fn parse_grid(buffer: impl BufRead) -> Grid {
    let mut values = Vec::new();
    let mut height = 0;
    for symbol in buffer.bytes() {
        match symbol.unwrap() {
            b'\r' => (),
            b'\n' => height += 1,
            v => values.push(v - b'0'),
        }
    }
    Grid {
        width: values.len() / height,
        height,
        values,
    }
}

#[test]
fn example_test() {
    let buffer = r#"1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581
"#
    .as_bytes();
    assert_eq!(calculate_lowest_total_risk(buffer), 40);
}
