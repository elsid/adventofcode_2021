use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{:?}", reboot_reactor(std::io::stdin().lock()));
}

fn reboot_reactor(buffer: impl BufRead) -> (usize, usize) {
    let cubes: Vec<Cube> = buffer.lines().map(|v| parse_cube(&v.unwrap())).collect();
    (init_reactor_reboot(&cubes), full_reactor_reboot(&cubes))
}

fn init_reactor_reboot(cubes: &[Cube]) -> usize {
    let mut reactor: Vec<State> = std::iter::repeat(State::None)
        .take(101 * 101 * 101)
        .collect();
    for cube in cubes.iter() {
        let ranges: [_; 3] = [
            cube.aabb.lower[0].max(-50)..=cube.aabb.upper[0].min(50),
            cube.aabb.lower[1].max(-50)..=cube.aabb.upper[1].min(50),
            cube.aabb.lower[2].max(-50)..=cube.aabb.upper[2].min(50),
        ];
        for x in ranges[0].clone() {
            for y in ranges[1].clone() {
                for z in ranges[2].clone() {
                    reactor[((z + 50) * 101 * 101 + (y + 50) * 101 + x - 50) as usize] = cube.state;
                }
            }
        }
    }
    reactor.iter().filter(|v| matches!(v, State::On)).count()
}

fn full_reactor_reboot(cubes: &[Cube]) -> usize {
    let mut non_intersecting_cubes = Vec::new();
    for cube in cubes.iter().rev() {
        add_non_intersecting(cube, &mut non_intersecting_cubes);
    }
    non_intersecting_cubes
        .iter()
        .filter(|v| matches!(v.state, State::On))
        .map(|v| get_area(&v.aabb.lower, &v.aabb.upper))
        .sum::<i64>() as usize
}

fn add_non_intersecting(cube: &Cube, non_intersecting_cubes: &mut Vec<Cube>) {
    let mut aabbs = vec![cube.aabb.clone()];
    while let Some(aabb) = aabbs.pop() {
        if let Some(i) = non_intersecting_cubes
            .iter()
            .enumerate()
            .find(|(_, v)| has_intersection(&aabb, &v.aabb))
            .map(|(i, _)| i)
        {
            let (lower, upper) = get_intersection(&aabb, &non_intersecting_cubes[i].aabb);
            if aabb.lower[0] < lower[0] {
                aabbs.push(Aabb {
                    lower: [aabb.lower[0], aabb.lower[1], aabb.lower[2]],
                    upper: [lower[0] - 1, aabb.upper[1], aabb.upper[2]],
                });
            }
            if upper[0] < aabb.upper[0] {
                aabbs.push(Aabb {
                    lower: [upper[0] + 1, aabb.lower[1], aabb.lower[2]],
                    upper: [aabb.upper[0], aabb.upper[1], aabb.upper[2]],
                });
            }
            if aabb.lower[1] < lower[1] {
                aabbs.push(Aabb {
                    lower: [aabb.lower[0], aabb.lower[1], aabb.lower[2]],
                    upper: [aabb.upper[0], lower[1] - 1, aabb.upper[2]],
                });
            }
            if upper[1] < aabb.upper[1] {
                aabbs.push(Aabb {
                    lower: [aabb.lower[0], upper[1] + 1, aabb.lower[2]],
                    upper: [aabb.upper[0], aabb.upper[1], aabb.upper[2]],
                });
            }
            if aabb.lower[2] < lower[2] {
                aabbs.push(Aabb {
                    lower: [aabb.lower[0], aabb.lower[1], aabb.lower[2]],
                    upper: [aabb.upper[0], aabb.upper[1], lower[2] - 1],
                });
            }
            if upper[2] < aabb.upper[2] {
                aabbs.push(Aabb {
                    lower: [aabb.lower[0], aabb.lower[1], upper[2] + 1],
                    upper: [aabb.upper[0], aabb.upper[1], aabb.upper[2]],
                });
            }
        } else {
            non_intersecting_cubes.push(Cube {
                state: cube.state,
                aabb,
            });
        }
    }
}

fn has_intersection(a: &Aabb, b: &Aabb) -> bool {
    for i in 0..3 {
        if a.upper[i] < b.lower[i] || b.upper[i] < a.lower[i] {
            return false;
        }
    }
    true
}

fn get_intersection(a: &Aabb, b: &Aabb) -> (Vec3, Vec3) {
    let mut lower = a.lower;
    let mut upper = a.upper;
    for i in 0..3 {
        lower[i] = lower[i].max(b.lower[i]);
        upper[i] = upper[i].min(b.upper[i]);
    }
    (lower, upper)
}

fn get_area(min: &Vec3, max: &Vec3) -> i64 {
    let mut result = 1;
    for i in 0..3 {
        result *= max[i] - min[i] + 1;
    }
    result
}

type Vec3 = [i64; 3];

#[derive(Copy, Clone)]
enum State {
    None,
    On,
    Off,
}

#[derive(Clone)]
struct Aabb {
    lower: Vec3,
    upper: Vec3,
}

struct Cube {
    state: State,
    aabb: Aabb,
}

fn parse_cube(line: &str) -> Cube {
    let (state, tail) = line.split_once(" ").unwrap();
    let mut lower = [0; 3];
    let mut upper = [0; 3];
    for v in tail.split(',') {
        let (coordinate, ranges) = v.split_once("=").unwrap();
        let (min_str, max_str) = ranges.split_once("..").unwrap();
        let min = i64::from_str(min_str).unwrap();
        let max = i64::from_str(max_str).unwrap();
        match coordinate {
            "x" => {
                lower[0] = min;
                upper[0] = max;
            }
            "y" => {
                lower[1] = min;
                upper[1] = max;
            }
            "z" => {
                lower[2] = min;
                upper[2] = max;
            }
            _ => (),
        }
    }
    Cube {
        state: match state {
            "on" => State::On,
            "off" => State::Off,
            _ => State::None,
        },
        aabb: Aabb { lower, upper },
    }
}

#[test]
fn small_example_test() {
    let buffer = r#"on x=10..12,y=10..12,z=10..12
on x=11..13,y=11..13,z=11..13
off x=9..11,y=9..11,z=9..11
on x=10..10,y=10..10,z=10..10
"#
    .as_bytes();
    assert_eq!(reboot_reactor(buffer), (39, 39));
}

#[test]
fn example_1_test() {
    let buffer = r#"on x=-20..26,y=-36..17,z=-47..7
on x=-20..33,y=-21..23,z=-26..28
on x=-22..28,y=-29..23,z=-38..16
on x=-46..7,y=-6..46,z=-50..-1
on x=-49..1,y=-3..46,z=-24..28
on x=2..47,y=-22..22,z=-23..27
on x=-27..23,y=-28..26,z=-21..29
on x=-39..5,y=-6..47,z=-3..44
on x=-30..21,y=-8..43,z=-13..34
on x=-22..26,y=-27..20,z=-29..19
off x=-48..-32,y=26..41,z=-47..-37
on x=-12..35,y=6..50,z=-50..-2
off x=-48..-32,y=-32..-16,z=-15..-5
on x=-18..26,y=-33..15,z=-7..46
off x=-40..-22,y=-38..-28,z=23..41
on x=-16..35,y=-41..10,z=-47..6
off x=-32..-23,y=11..30,z=-14..3
on x=-49..-5,y=-3..45,z=-29..18
off x=18..30,y=-20..-8,z=-3..13
on x=-41..9,y=-7..43,z=-33..15
on x=-54112..-39298,y=-85059..-49293,z=-27449..7877
on x=967..23432,y=45373..81175,z=27513..53682
"#
    .as_bytes();
    assert_eq!(reboot_reactor(buffer), (590784, 39769202357779));
}

#[test]
fn example_2_test() {
    let buffer = r#"on x=-5..47,y=-31..22,z=-19..33
on x=-44..5,y=-27..21,z=-14..35
on x=-49..-1,y=-11..42,z=-10..38
on x=-20..34,y=-40..6,z=-44..1
off x=26..39,y=40..50,z=-2..11
on x=-41..5,y=-41..6,z=-36..8
off x=-43..-33,y=-45..-28,z=7..25
on x=-33..15,y=-32..19,z=-34..11
off x=35..47,y=-46..-34,z=-11..5
on x=-14..36,y=-6..44,z=-16..29
on x=-57795..-6158,y=29564..72030,z=20435..90618
on x=36731..105352,y=-21140..28532,z=16094..90401
on x=30999..107136,y=-53464..15513,z=8553..71215
on x=13528..83982,y=-99403..-27377,z=-24141..23996
on x=-72682..-12347,y=18159..111354,z=7391..80950
on x=-1060..80757,y=-65301..-20884,z=-103788..-16709
on x=-83015..-9461,y=-72160..-8347,z=-81239..-26856
on x=-52752..22273,y=-49450..9096,z=54442..119054
on x=-29982..40483,y=-108474..-28371,z=-24328..38471
on x=-4958..62750,y=40422..118853,z=-7672..65583
on x=55694..108686,y=-43367..46958,z=-26781..48729
on x=-98497..-18186,y=-63569..3412,z=1232..88485
on x=-726..56291,y=-62629..13224,z=18033..85226
on x=-110886..-34664,y=-81338..-8658,z=8914..63723
on x=-55829..24974,y=-16897..54165,z=-121762..-28058
on x=-65152..-11147,y=22489..91432,z=-58782..1780
on x=-120100..-32970,y=-46592..27473,z=-11695..61039
on x=-18631..37533,y=-124565..-50804,z=-35667..28308
on x=-57817..18248,y=49321..117703,z=5745..55881
on x=14781..98692,y=-1341..70827,z=15753..70151
on x=-34419..55919,y=-19626..40991,z=39015..114138
on x=-60785..11593,y=-56135..2999,z=-95368..-26915
on x=-32178..58085,y=17647..101866,z=-91405..-8878
on x=-53655..12091,y=50097..105568,z=-75335..-4862
on x=-111166..-40997,y=-71714..2688,z=5609..50954
on x=-16602..70118,y=-98693..-44401,z=5197..76897
on x=16383..101554,y=4615..83635,z=-44907..18747
off x=-95822..-15171,y=-19987..48940,z=10804..104439
on x=-89813..-14614,y=16069..88491,z=-3297..45228
on x=41075..99376,y=-20427..49978,z=-52012..13762
on x=-21330..50085,y=-17944..62733,z=-112280..-30197
on x=-16478..35915,y=36008..118594,z=-7885..47086
off x=-98156..-27851,y=-49952..43171,z=-99005..-8456
off x=2032..69770,y=-71013..4824,z=7471..94418
on x=43670..120875,y=-42068..12382,z=-24787..38892
off x=37514..111226,y=-45862..25743,z=-16714..54663
off x=25699..97951,y=-30668..59918,z=-15349..69697
off x=-44271..17935,y=-9516..60759,z=49131..112598
on x=-61695..-5813,y=40978..94975,z=8655..80240
off x=-101086..-9439,y=-7088..67543,z=33935..83858
off x=18020..114017,y=-48931..32606,z=21474..89843
off x=-77139..10506,y=-89994..-18797,z=-80..59318
off x=8476..79288,y=-75520..11602,z=-96624..-24783
on x=-47488..-1262,y=24338..100707,z=16292..72967
off x=-84341..13987,y=2429..92914,z=-90671..-1318
off x=-37810..49457,y=-71013..-7894,z=-105357..-13188
off x=-27365..46395,y=31009..98017,z=15428..76570
off x=-70369..-16548,y=22648..78696,z=-1892..86821
on x=-53470..21291,y=-120233..-33476,z=-44150..38147
off x=-93533..-4276,y=-16170..68771,z=-104985..-24507
"#
    .as_bytes();
    assert_eq!(reboot_reactor(buffer), (474140, 2758514936282235));
}
