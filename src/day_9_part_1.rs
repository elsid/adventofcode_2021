use std::io::BufRead;

fn main() {
    println!("{}", calculate_total_risk(std::io::stdin().lock()));
}

fn calculate_total_risk(buffer: impl BufRead) -> u64 {
    let heightmap = parse_heightmap(buffer);
    let mut risk = 0;
    for y in 0..heightmap.height {
        for x in 0..heightmap.width {
            if let Some(height) = get_low_point(&heightmap, x, y) {
                risk += 1 + height as u64
            }
        }
    }
    risk
}

fn get_low_point(heightmap: &Heightmap, x: usize, y: usize) -> Option<u8> {
    let value = heightmap.value(x, y);
    if x > 0 && heightmap.value(x - 1, y) <= value {
        return None;
    }
    if y > 0 && heightmap.value(x, y - 1) <= value {
        return None;
    }
    if y + 1 < heightmap.height && heightmap.value(x, y + 1) <= value {
        return None;
    }
    if x + 1 < heightmap.width && heightmap.value(x + 1, y) <= value {
        return None;
    }
    Some(value)
}

struct Heightmap {
    values: Vec<u8>,
    width: usize,
    height: usize,
}

impl Heightmap {
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    fn value(&self, x: usize, y: usize) -> u8 {
        self.values[self.index(x, y)]
    }
}

fn parse_heightmap(buffer: impl BufRead) -> Heightmap {
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
    assert_eq!(calculate_total_risk(buffer), 15);
}
