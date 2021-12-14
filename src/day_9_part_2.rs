use std::io::BufRead;

fn main() {
    println!("{}", product_basin_top_3_sizes(std::io::stdin().lock()));
}

fn product_basin_top_3_sizes(buffer: impl BufRead) -> u64 {
    let heightmap = parse_heightmap(buffer);
    let mut basins = Heightmap {
        values: std::iter::repeat(0).take(heightmap.values.len()).collect(),
        width: heightmap.width,
        height: heightmap.height,
    };
    let mut basin_number = 1;
    for y in 0..heightmap.height {
        for x in 0..heightmap.width {
            if is_low_point(&heightmap, x, y) {
                fill_basin(basin_number, x, y, &heightmap, &mut basins);
                basin_number += 1;
            }
        }
    }
    let mut basin_sizes: Vec<u64> = std::iter::repeat(0)
        .take(basin_number as usize - 1)
        .collect();
    for basin in basins.values.iter() {
        if *basin != 0 {
            basin_sizes[*basin as usize - 1] += 1;
        }
    }
    basin_sizes.sort_unstable();
    basin_sizes.iter().rev().take(3).product()
}

fn fill_basin(
    value: u64,
    start_x: usize,
    start_y: usize,
    heightmap: &Heightmap<u8>,
    basins: &mut Heightmap<u64>,
) {
    let mut positions = vec![(start_x, start_y)];
    let try_push = |x, y, height, basins: &Heightmap<u64>, positions: &mut Vec<(usize, usize)>| {
        let neighbor_height = *heightmap.value(x, y);
        if neighbor_height != 9 && neighbor_height > height && *basins.value(x, y) == 0 {
            positions.push((x, y));
        }
    };
    while let Some((x, y)) = positions.pop() {
        basins.set_value(x, y, value);
        let height = *heightmap.value(x, y);
        if x > 0 {
            try_push(x - 1, y, height, basins, &mut positions);
        }
        if y > 0 {
            try_push(x, y - 1, height, basins, &mut positions);
        }
        if y + 1 < heightmap.height {
            try_push(x, y + 1, height, basins, &mut positions);
        }
        if x + 1 < heightmap.width {
            try_push(x + 1, y, height, basins, &mut positions);
        }
    }
}

fn is_low_point<T: PartialOrd>(heightmap: &Heightmap<T>, x: usize, y: usize) -> bool {
    let value = heightmap.value(x, y);
    if x > 0 && heightmap.value(x - 1, y) <= value {
        return false;
    }
    if y > 0 && heightmap.value(x, y - 1) <= value {
        return false;
    }
    if y + 1 < heightmap.height && heightmap.value(x, y + 1) <= value {
        return false;
    }
    if x + 1 < heightmap.width && heightmap.value(x + 1, y) <= value {
        return false;
    }
    true
}

struct Heightmap<T> {
    values: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Heightmap<T> {
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    fn value(&self, x: usize, y: usize) -> &T {
        &self.values[self.index(x, y)]
    }

    fn set_value(&mut self, x: usize, y: usize, value: T) {
        let index = self.index(x, y);
        self.values[index] = value
    }
}

fn parse_heightmap(buffer: impl BufRead) -> Heightmap<u8> {
    let mut values = Vec::new();
    let mut height = 0;
    for symbol in buffer.bytes() {
        match symbol.unwrap() {
            b'\r' => (),
            b'\n' => height += 1,
            v => values.push(v - b'0'),
        }
    }
    Heightmap {
        width: values.len() / height,
        height,
        values,
    }
}

#[test]
fn example_test() {
    let buffer = r#"2199943210
3987894921
9856789892
8767896789
9899965678
"#
    .as_bytes();
    assert_eq!(product_basin_top_3_sizes(buffer), 1134);
}
