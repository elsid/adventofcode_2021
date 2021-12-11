use std::io::BufRead;

fn main() {
    println!("{}", count_octopus_flashes(std::io::stdin().lock()));
}

const MAX_STEPS: usize = 100;
const FLASH_ENERGY: u8 = 9;

fn count_octopus_flashes(buffer: impl BufRead) -> usize {
    let mut octopus_grid = parse_octopus_grid(buffer);
    let mut flashes = 0;
    for _ in 0..MAX_STEPS {
        flashes += update_octopus_energy(&mut octopus_grid);
    }
    flashes
}

fn update_octopus_energy(octopus_grid: &mut OctopusGrid) -> usize {
    let mut new_flashes = Vec::new();
    let mut flashes = 0;
    for (index, energy) in octopus_grid.energy.iter_mut().enumerate() {
        *energy += 1;
        if *energy > FLASH_ENERGY {
            *energy = 0;
            new_flashes.push(index);
            flashes += 1;
        }
    }
    while let Some(flash_index) = new_flashes.pop() {
        let (flash_x, flash_y) = octopus_grid.position(flash_index);
        for y in (flash_y - 1)..=(flash_y + 1) {
            for x in (flash_x - 1)..=(flash_x + 1) {
                if !octopus_grid.is_within_borders(x, y) {
                    continue;
                }
                let index = octopus_grid.index(x, y);
                if octopus_grid.energy[index] == 0 {
                    continue;
                }
                octopus_grid.energy[index] += 1;
                if octopus_grid.energy[index] <= FLASH_ENERGY {
                    continue;
                }
                octopus_grid.energy[index] = 0;
                new_flashes.push(index);
                flashes += 1;
            }
        }
    }
    flashes
}

struct OctopusGrid {
    energy: Vec<u8>,
    width: usize,
    height: usize,
}

impl OctopusGrid {
    fn position(&self, index: usize) -> (isize, isize) {
        (
            (index % self.width) as isize,
            (index / self.width as usize) as isize,
        )
    }

    fn index(&self, x: isize, y: isize) -> usize {
        x as usize + y as usize * self.width
    }

    fn is_within_borders(&self, x: isize, y: isize) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }
}

fn parse_octopus_grid(buffer: impl BufRead) -> OctopusGrid {
    let mut values = Vec::new();
    let mut height = 0;
    for symbol in buffer.bytes() {
        match symbol.unwrap() {
            b'\r' => (),
            b'\n' => height += 1,
            v => values.push(v - b'0'),
        }
    }
    OctopusGrid {
        width: values.len() / height,
        height,
        energy: values,
    }
}

#[test]
fn example_test() {
    let buffer = r#"5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526
"#
    .as_bytes();
    assert_eq!(count_octopus_flashes(buffer), 1656);
}
