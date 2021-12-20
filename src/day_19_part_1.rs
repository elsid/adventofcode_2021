use std::collections::{BTreeSet, HashMap};
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{}", count_beacons(std::io::stdin().lock()));
}

fn count_beacons(buffer: impl BufRead) -> usize {
    let mut scanners = parse_scanners(buffer);
    let (head, tail) = scanners.split_at_mut(1);
    let first = &mut head[0];
    loop {
        let mut aggregated = false;
        for scanner in tail.iter_mut() {
            if scanner.is_empty() {
                continue;
            }
            if let Some(transform) = find_relative_transformation(first, scanner, 12) {
                for pos in scanner.iter() {
                    first.push(apply_transform(*pos, &transform));
                }
                first.sort_unstable();
                first.dedup();
                scanner.clear();
                aggregated = true;
            }
        }
        if !aggregated {
            break;
        }
    }
    first.len()
}

fn apply_transform(vec: Vec3, transform: &Transform) -> Vec3 {
    add_vec3(mat3_vec3_product(&transform.rot, vec), transform.shift)
}

type Vec3 = [i16; 3];
type Mat3 = [Vec3; 3];

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
struct Transform {
    shift: Vec3,
    rot: Mat3,
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
                .map(|v| i16::from_str(v).unwrap())
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
    for rot in generate_rotations() {
        let shifts = find_shifts(dst, src, &rot);
        if let Some(shift) = find_overlap(&shifts, src.len(), min_count) {
            return Some(Transform { shift, rot });
        }
    }
    None
}

fn find_shifts(src: &[Vec3], dst: &[Vec3], rot: &Mat3) -> Vec<Vec3> {
    let mut shifts = Vec::with_capacity(dst.len() * src.len());
    for src_pos in src {
        for dst_pos in dst {
            shifts.push(sub_vec3(*dst_pos, mat3_vec3_product(rot, *src_pos)));
        }
    }
    shifts
}

fn find_overlap(shifts: &[Vec3], width: usize, min_count: usize) -> Option<Vec3> {
    let mut candidates: HashMap<Vec3, usize> = HashMap::new();
    for i in 0..shifts.len() / width {
        for j in 0..width {
            let shift = shifts[j + i * width];
            let count = candidates.entry(shift).or_default();
            *count += 1;
            if *count >= min_count {
                return Some(shift);
            }
        }
    }
    None
}

fn generate_rotations() -> [Mat3; 24] {
    let mut unique = BTreeSet::new();
    for rot_x in 0u8..4 {
        for rot_y in 0u8..4 {
            for rot_z in 0u8..4 {
                unique.insert(make_rotation_matrix(rot_x, rot_y, rot_z));
            }
        }
    }
    let mut result: [Mat3; 24] = [[[0; 3]; 3]; 24];
    for (i, v) in unique.iter().enumerate() {
        result[i] = *v;
    }
    result
}

fn make_rotation_matrix(rot_x: u8, rot_y: u8, rot_z: u8) -> Mat3 {
    let x: Mat3 = [
        [1, 0, 0],
        [0, cos(rot_x), -sin(rot_x)],
        [0, sin(rot_x), cos(rot_x)],
    ];
    let y: Mat3 = [
        [cos(rot_y), 0, sin(rot_y)],
        [0, 1, 0],
        [-sin(rot_y), 0, cos(rot_y)],
    ];
    let z: Mat3 = [
        [cos(rot_z), -sin(rot_z), 0],
        [sin(rot_z), cos(rot_z), 0],
        [0, 0, 1],
    ];
    mat3_product(&mat3_product(&z, &y), &x)
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

fn dot_product(a: Vec3, b: Vec3) -> i16 {
    a.iter().zip(b.iter()).map(|(a, b)| *a * *b).sum()
}

fn cos(v: u8) -> i16 {
    match v {
        0 => 1,
        2 => -1,
        _ => 0,
    }
}

fn sin(v: u8) -> i16 {
    match v {
        1 => 1,
        3 => -1,
        _ => 0,
    }
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
    assert_eq!(count_beacons(buffer), 79);
}
