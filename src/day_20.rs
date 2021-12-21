use image::{ImageBuffer, Rgb};
use std::collections::BTreeMap;
use std::io::BufRead;

fn main() {
    if std::env::args()
        .nth(1)
        .map(|v| v == "images")
        .unwrap_or(false)
    {
        println!("generating images...");
        generate_images(24 * 10, std::io::stdin().lock());
    } else {
        println!("{:?}", count_enhanced_light_pixels(std::io::stdin().lock()));
    }
}

fn generate_images(number: usize, buffer: impl BufRead) {
    let (enhancement, mut image) = parse_image(buffer);
    let mut images = Vec::with_capacity(number);
    let mut default_pixel = '0';
    images.push((image.clone(), default_pixel));
    for _ in 0..number {
        image = enhance_image(&enhancement, &image, &mut default_pixel);
        images.push((image.clone(), default_pixel));
    }
    let (min_x, min_y) = *images.last().unwrap().0.keys().next().unwrap();
    let (max_x, max_y) = *images.last().unwrap().0.keys().rev().next().unwrap();
    for (n, (image, default_pixel)) in images.iter().enumerate() {
        save_image(n, min_x, min_y, max_x, max_y, image, *default_pixel);
    }
}

fn save_image(
    n: usize,
    min_x: isize,
    min_y: isize,
    max_x: isize,
    max_y: isize,
    image: &BTreeMap<(isize, isize), u8>,
    default_pixel: char,
) {
    let mut buffer = ImageBuffer::new((max_x - min_x) as u32 + 1, (max_y - min_y) as u32 + 1);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            buffer.put_pixel(
                (x - min_x) as u32,
                (y - min_y) as u32,
                match image.get(&(x, y)) {
                    Some(b'#') => Rgb([192, 192, 192]),
                    Some(b'.') => Rgb([128u8, 128, 128]),
                    _ => match default_pixel {
                        '1' => Rgb([192u8, 192, 192]),
                        _ => Rgb([128u8, 128, 128]),
                    },
                },
            );
        }
    }
    buffer.save(format!("images/day_20_{:03}.png", n)).unwrap();
}

const FIRST_LIMIT: usize = 2;
const SECOND_LIMIT: usize = 50;

fn count_enhanced_light_pixels(buffer: impl BufRead) -> (usize, usize) {
    let (enhancement, mut image) = parse_image(buffer);
    let mut default_pixel = '0';
    for _ in 0..FIRST_LIMIT {
        image = enhance_image(&enhancement, &image, &mut default_pixel);
    }
    let first = image.values().filter(|v| **v == b'#').count();
    for _ in FIRST_LIMIT..SECOND_LIMIT {
        image = enhance_image(&enhancement, &image, &mut default_pixel);
    }
    (first, image.values().filter(|v| **v == b'#').count())
}

fn enhance_image(
    enhancement: &[u8],
    image: &BTreeMap<(isize, isize), u8>,
    default_pixel: &mut char,
) -> BTreeMap<(isize, isize), u8> {
    let mut result = BTreeMap::new();
    for (x, y) in image.keys() {
        result.insert(
            (*x, *y),
            enhance_pixel(*x, *y, *default_pixel, enhancement, image),
        );
    }
    let (min_x, min_y) = *image.keys().next().unwrap();
    let (max_x, max_y) = *image.keys().rev().next().unwrap();
    let mut shift = 1;
    loop {
        let mut light_count = 0;
        let mut count = 0;
        let mut add_pixel = |x, y| {
            let value = enhance_pixel(x, y, *default_pixel, enhancement, image);
            result.insert((x, y), value);
            light_count += (value == b'#') as usize;
            count += 1;
        };
        for x in min_x - shift..=max_x + shift {
            for y in [min_y - shift, max_y + shift] {
                add_pixel(x, y);
            }
        }
        for y in min_y - shift..=max_y + shift {
            for x in [min_x - shift, max_x + shift] {
                add_pixel(x, y);
            }
        }
        if light_count == 0 {
            *default_pixel = '0';
            break;
        }
        if light_count == count {
            *default_pixel = '1';
            break;
        }
        shift += 1;
    }
    result
}

fn enhance_pixel(
    x: isize,
    y: isize,
    default_value: char,
    enhancement: &[u8],
    image: &BTreeMap<(isize, isize), u8>,
) -> u8 {
    let mut pixels = String::new();
    for py in y - 1..=y + 1 {
        for px in x - 1..=x + 1 {
            match image.get(&(px, py)) {
                Some(b'#') => pixels.push('1'),
                Some(b'.') => pixels.push('0'),
                _ => pixels.push(default_value),
            }
        }
    }
    enhancement[usize::from_str_radix(&pixels, 2).unwrap()]
}

fn parse_image(buffer: impl BufRead) -> (Vec<u8>, BTreeMap<(isize, isize), u8>) {
    let mut enhancement = Vec::new();
    let mut image = BTreeMap::new();
    let mut y = 0isize;
    for line in buffer.lines() {
        let line_str = line.unwrap();
        if enhancement.is_empty() {
            enhancement = line_str.as_bytes().iter().copied().collect();
        } else if !line_str.is_empty() {
            for (x, byte) in line_str.as_bytes().iter().enumerate() {
                image.insert((x as isize, y), *byte);
            }
            y += 1;
        }
    }
    (enhancement, image)
}

#[test]
fn example_test() {
    let buffer = r#"..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###
"#
    .as_bytes();
    assert_eq!(count_enhanced_light_pixels(buffer), (35, 3351));
}
