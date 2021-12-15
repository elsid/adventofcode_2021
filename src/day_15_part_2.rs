use std::collections::BinaryHeap;
use std::io::BufRead;

fn main() {
    println!("{}", calculate_lowest_total_risk(std::io::stdin().lock()));
}

fn calculate_lowest_total_risk(buffer: impl BufRead) -> i64 {
    let grid_tile = parse_grid(buffer);
    let mut grid = Grid {
        values: std::iter::repeat(0)
            .take(grid_tile.values.len() * 25)
            .collect(),
        width: grid_tile.width * 5,
        height: grid_tile.height * 5,
    };
    for tile_y in 0..5 {
        for tile_x in 0..5 {
            for y in 0..grid_tile.height {
                for x in 0..grid_tile.width {
                    let grid_index =
                        grid.index(grid_tile.width * tile_x + x, grid_tile.height * tile_y + y);
                    let grid_tile_index = grid_tile.index(x, y);
                    grid.values[grid_index] = (((grid_tile.values[grid_tile_index] as u64
                        + tile_x as u64
                        + tile_y as u64)
                        - 1)
                        % 9
                        + 1) as u8;
                }
            }
        }
    }
    find_min_risk(&grid)
}

fn find_min_risk(grid: &Grid) -> i64 {
    type Nodes = BinaryHeap<(i64, usize)>;
    let mut visited: Vec<bool> = std::iter::repeat(false).take(grid.values.len()).collect();
    visited[0] = true;
    let mut try_push = |risk, x, y, nodes: &mut Nodes| {
        let index = grid.index(x, y);
        if visited[index] {
            return;
        }
        nodes.push(((risk - grid.values[index] as i64), index));
        visited[index] = true;
    };
    let mut nodes = Nodes::new();
    nodes.push((0, 0));
    while let Some((node_risk, node_index)) = nodes.pop() {
        if node_index == grid.values.len() - 1 {
            return -node_risk;
        }
        let (node_x, node_y) = grid.position(node_index);
        if node_x > 0 {
            try_push(node_risk, node_x - 1, node_y, &mut nodes);
        }
        if node_y > 0 {
            try_push(node_risk, node_x, node_y - 1, &mut nodes);
        }
        if node_x + 1 < grid.width {
            try_push(node_risk, node_x + 1, node_y, &mut nodes);
        }
        if node_y + 1 < grid.height {
            try_push(node_risk, node_x, node_y + 1, &mut nodes);
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
    assert_eq!(calculate_lowest_total_risk(buffer), 315);
}
