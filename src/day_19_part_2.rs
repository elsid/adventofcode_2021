use std::collections::BTreeMap;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!(
        "{}",
        find_max_distance_between_scanners(std::io::stdin().lock())
    );
}

fn find_max_distance_between_scanners(buffer: impl BufRead) -> i64 {
    let mut scanners = parse_scanners(buffer);
    let (transforms, predecessors) = find_transforms(&mut scanners);
    let mut absolute_scanners = vec![[0; 3]];
    for i in 1..scanners.len() {
        let mut pos = [0; 3];
        let mut current = i;
        loop {
            let prev = predecessors[current];
            if prev == usize::MAX {
                break;
            }
            pos = apply_transform(pos, transforms.get(&(current, prev)).unwrap());
            current = prev;
        }
        absolute_scanners.push(pos);
    }
    let mut max_distance = 0;
    for i in 0..absolute_scanners.len() - 1 {
        for other in &absolute_scanners[i + 1..absolute_scanners.len()] {
            max_distance = max_distance.max(get_manhattan_distance(absolute_scanners[i], *other));
        }
    }
    max_distance
}

fn find_transforms(
    scanners: &mut Vec<Vec<Vec3>>,
) -> (BTreeMap<(usize, usize), Transform>, Vec<usize>) {
    let mut transforms = BTreeMap::new();
    let try_add_transform =
        |src: usize,
         dst: usize,
         scanners: &mut Vec<Vec<Vec3>>,
         transforms: &mut BTreeMap<(usize, usize), Transform>| {
            if let Some(transform) =
                find_relative_transformation(&scanners[src], &scanners[dst], 12)
            {
                for k in 0..scanners[src].len() {
                    let pos = scanners[src][k];
                    scanners[dst].push(apply_transform(pos, &transform));
                }
                scanners[dst].sort_unstable();
                scanners[dst].dedup();
                transforms.insert((src, dst), transform);
                true
            } else {
                false
            }
        };
    let mut predecessors = build_shortest_transformation_paths(0, scanners.len(), &transforms);
    if predecessors.iter().skip(1).all(|v| *v != usize::MAX) {
        return (transforms, predecessors);
    }
    for i in 0..scanners.len() - 1 {
        for j in i + 1..scanners.len() {
            for (src, dst) in [(i, j), (j, i)] {
                if try_add_transform(src, dst, scanners, &mut transforms) {
                    predecessors =
                        build_shortest_transformation_paths(0, scanners.len(), &transforms);
                    if predecessors.iter().skip(1).all(|v| *v != usize::MAX) {
                        return (transforms, predecessors);
                    }
                }
            }
        }
    }
    (transforms, predecessors)
}

fn get_manhattan_distance(a: Vec3, b: Vec3) -> i64 {
    sub_vec3(a, b).iter().map(|v| v.abs()).sum()
}

fn build_shortest_transformation_paths(
    src: usize,
    scanners: usize,
    transforms: &BTreeMap<(usize, usize), Transform>,
) -> Vec<usize> {
    let mut predecessors: Vec<usize> = std::iter::repeat(usize::MAX).take(scanners).collect();
    let mut distances: Vec<usize> = std::iter::repeat(usize::MAX - 1).take(scanners).collect();
    distances[src] = 0;
    for _ in 0..scanners - 1 {
        for ((v, u), _) in transforms.iter() {
            if distances[*u] + 1 < distances[*v] {
                distances[*v] = distances[*u] + 1;
                predecessors[*v] = *u;
            }
        }
    }
    predecessors
}

fn apply_transform(vec: Vec3, transform: &Transform) -> Vec3 {
    add_vec3(rotate_vec3(vec, transform.rot), transform.shift)
}

type Vec3 = [i64; 3];
type Rot3 = [u8; 3];
type Mat3 = [Vec3; 3];

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct Transform {
    shift: Vec3,
    rot: Rot3,
}

fn parse_scanners(buffer: impl BufRead) -> Vec<Vec<Vec3>> {
    let mut scanners: Vec<Vec<Vec3>> = Vec::new();
    for line in buffer.lines() {
        let line_str = line.unwrap();
        if line_str.starts_with("--- ") {
            scanners.push(Vec::new());
        } else if !line_str.is_empty() {
            let mut position = [0; 3];
            for (i, coordinate) in line_str
                .splitn(3, ',')
                .map(|v| i64::from_str(v).unwrap())
                .enumerate()
            {
                position[i] = coordinate;
            }
            scanners.last_mut().unwrap().push(position);
        }
    }
    scanners
}

fn find_relative_transformation(src: &[Vec3], dst: &[Vec3], min_count: usize) -> Option<Transform> {
    let mut candidates: BTreeMap<Transform, usize> = BTreeMap::new();
    for rot in generate_rotations() {
        for src_pos in src {
            let rotated_src_pos = rotate_vec3(*src_pos, rot);
            for dst_pos in dst {
                let shift = sub_vec3(*dst_pos, rotated_src_pos);
                let count = candidates.entry(Transform { shift, rot }).or_default();
                *count += 1;
                if *count >= min_count {
                    return Some(Transform { shift, rot });
                }
            }
        }
    }
    None
}

fn generate_rotations() -> [Rot3; 24] {
    let mut result: [Rot3; 24] = [[0; 3]; 24];
    let mut i = 0;
    for rot_x in 0u8..4 {
        for rot_y in 0u8..3 {
            for rot_z in 0u8..2 {
                result[i] = [rot_x, rot_y, rot_z];
                i += 1;
            }
        }
    }
    result
}

fn add_vec3(mut a: Vec3, b: Vec3) -> Vec3 {
    for i in 0..3 {
        a[i] += b[i];
    }
    a
}

fn sub_vec3(mut a: Vec3, b: Vec3) -> Vec3 {
    for i in 0..3 {
        a[i] -= b[i];
    }
    a
}

fn rotate_vec3(vec: Vec3, rot: Rot3) -> Vec3 {
    let x: Mat3 = [
        [1, 0, 0],
        [0, cos(rot[0]), -sin(rot[0])],
        [0, sin(rot[0]), cos(rot[0])],
    ];
    let y: Mat3 = [
        [cos(rot[1]), 0, sin(rot[1])],
        [0, 1, 0],
        [-sin(rot[1]), 0, cos(rot[1])],
    ];
    let z: Mat3 = [
        [cos(rot[2]), -sin(rot[2]), 0],
        [sin(rot[2]), cos(rot[2]), 0],
        [0, 0, 1],
    ];
    mat3_vec3_product(&mat3_product(&mat3_product(&z, &y), &x), vec)
}

fn mat3_vec3_product(mat: &Mat3, vec: Vec3) -> Vec3 {
    let mut result: Vec3 = [0; 3];
    for i in 0..3 {
        result[i] = dot_product(mat[i], vec);
    }
    result
}

fn mat3_product(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut result: Mat3 = [[0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for (k, bv) in b.iter().enumerate() {
                result[i][j] += a[i][k] * bv[j];
            }
        }
    }
    result
}

fn dot_product(a: Vec3, b: Vec3) -> i64 {
    a.iter().zip(b.iter()).map(|(a, b)| *a * *b).sum()
}

fn cos(v: u8) -> i64 {
    match v {
        0 => 1,
        2 => -1,
        _ => 0,
    }
}

fn sin(v: u8) -> i64 {
    match v {
        1 => 1,
        3 => -1,
        _ => 0,
    }
}

#[test]
fn find_relative_transformation_only_shift_test() {
    let src: &[Vec3] = &[[0, 2, 0], [4, 1, 0], [3, 3, 0]];
    let shift = [-5, -2, 0];
    let dst: Vec<Vec3> = src.iter().map(|v| add_vec3(*v, shift)).collect();
    assert_eq!(
        find_relative_transformation(src, &dst, 3),
        Some(Transform {
            shift,
            rot: [0, 0, 0],
        })
    );
}

#[test]
fn find_relative_transformation_only_rotation_test() {
    let src: &[Vec3] = &[[0, 2, 0], [4, 1, 0], [3, 3, 0]];
    let rot: Rot3 = [1, 0, 0];
    let dst: Vec<Vec3> = src.iter().map(|v| rotate_vec3(*v, rot)).collect();
    assert_eq!(
        find_relative_transformation(src, &dst, 3),
        Some(Transform {
            shift: [0, 0, 0],
            rot,
        })
    );
}

#[test]
fn find_relative_transformation_with_shift_and_rotation_test() {
    let src: &[Vec3] = &[[0, 2, 0], [4, 1, 0], [3, 3, 0]];
    let shift = [-5, -2, 0];
    let rot: Rot3 = [1, 0, 0];
    let dst: Vec<Vec3> = src
        .iter()
        .map(|v| add_vec3(rotate_vec3(*v, rot), shift))
        .collect();
    assert_eq!(
        find_relative_transformation(src, &dst, 3),
        Some(Transform { shift, rot })
    );
}

#[test]
fn example_test() {
    let buffer = r#"--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14
"#
    .as_bytes();
    assert_eq!(find_max_distance_between_scanners(buffer), 3621);
}
