use std::io::BufRead;

fn main() {
    println!("{:?}", move_sea_cucumbers(std::io::stdin().lock()));
}

fn move_sea_cucumbers(buffer: impl BufRead) -> usize {
    let mut grid = parse_grid(buffer);
    let mut step = 0;
    loop {
        step += 1;
        if [b'>', b'v']
            .iter()
            .map(|direction| move_half_step(*direction, &mut grid) as usize)
            .sum::<usize>()
            == 0
        {
            break;
        }
    }
    step
}

fn move_half_step(direction: u8, grid: &mut Grid) -> bool {
    let mut moved = grid.clone();
    moved.values.fill(0);
    let mut has_moved = false;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if moved.get(x, y) > 0 {
                continue;
            }
            let (next_x, next_y) = match direction {
                b'>' => ((x + 1) % grid.width, y),
                b'v' => (x, (y + 1) % grid.height),
                _ => unreachable!(),
            };
            if moved.get(next_x, next_y) > 0 {
                continue;
            }
            let current = grid.get(x, y);
            if current == direction && grid.get(next_x, next_y) == b'.' {
                grid.set(x, y, b'.');
                moved.set(x, y, 1);
                grid.set(next_x, next_y, current);
                moved.set(next_x, next_y, 1);
                has_moved = true;
            }
        }
    }
    has_moved
}

#[cfg(test)]
fn grid_to_string(grid: &Grid) -> String {
    let mut result = String::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            result.push(grid.get(x, y) as char);
        }
        result.push('\n');
    }
    result
}

fn parse_grid(buffer: impl BufRead) -> Grid {
    let mut x = 0;
    let mut height = 0;
    let mut width = 0;
    let mut values = Vec::new();
    for symbol in buffer.bytes() {
        match symbol.unwrap() {
            b'\n' => {
                width = width.max(x);
                height += 1;
                x = 0;
            }
            b'\r' => (),
            v => {
                x += 1;
                values.push(v);
            }
        }
    }
    Grid {
        width,
        height,
        values,
    }
}

#[derive(Clone)]
struct Grid {
    values: Vec<u8>,
    width: usize,
    height: usize,
}

impl Grid {
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.values[self.index(x, y)]
    }

    fn set(&mut self, x: usize, y: usize, value: u8) {
        let i = self.index(x, y);
        self.values[i] = value;
    }
}

#[test]
fn move_half_step_0_test() {
    let mut grid = parse_grid(
        r#"...>>>>>...
"#
        .as_bytes(),
    );
    move_half_step(b'>', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"...>>>>.>..
"#
    );
    move_half_step(b'>', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"...>>>.>.>.
"#
    );
}

#[test]
fn move_half_step_1_test() {
    let mut grid = parse_grid(
        r#"...>...
.......
......>
v.....>
......>
.......
..vvv..
"#
        .as_bytes(),
    );
    move_half_step(b'>', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"....>..
.......
>......
v.....>
>......
.......
..vvv..
"#
    );
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"..vv>..
.......
>......
v.....>
>......
.......
....v..
"#
    );
    move_half_step(b'>', &mut grid);
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"....v>.
..vv...
.>.....
......>
v>.....
.......
.......
"#
    );
    move_half_step(b'>', &mut grid);
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"......>
..v.v..
..>v...
>......
..>....
v......
.......
"#
    );
    move_half_step(b'>', &mut grid);
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#">......
..v....
..>.v..
.>.v...
...>...
.......
v......
"#
    );
}

#[test]
fn move_half_step_2_test() {
    let mut grid = parse_grid(
        r#"v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>
"#
        .as_bytes(),
    );
    move_half_step(b'>', &mut grid);
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#"....>.>v.>
v.v>.>v.v.
>v>>..>v..
>>v>v>.>.v
.>v.v...v.
v>>.>vvv..
..v...>>..
vv...>>vv.
>.v.v..v.v
"#
    );
    move_half_step(b'>', &mut grid);
    move_half_step(b'v', &mut grid);
    assert_eq!(
        grid_to_string(&grid),
        r#">.v.v>>..v
v.v.>>vv..
>v>.>.>.v.
>>v>v.>v>.
.>..v....v
.>v>>.v.v.
v....v>v>.
.vv..>>v..
v>.....vv.
"#
    );
}

#[test]
fn example_test() {
    let buffer = r#"v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>
"#
    .as_bytes();
    assert_eq!(move_sea_cucumbers(buffer), 58);
}
