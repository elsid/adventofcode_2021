use std::io::{BufRead, Read};

fn main() {
    println!("{}", evaluate_packet(std::io::stdin().lock()));
}

fn evaluate_packet(buffer: impl BufRead) -> u64 {
    buffer
        .lines()
        .map(Result::unwrap)
        .map(|v| decode_hex_packet(&v))
        .map(|v| decode_bin_packet(&mut v.as_bytes()))
        .map(|v| evaluate(&v))
        .sum()
}

fn evaluate(packet: &Packet) -> u64 {
    match &packet.data {
        Expression::Sum(v) => v.iter().map(evaluate).sum(),
        Expression::Product(v) => v.iter().map(evaluate).product(),
        Expression::Minimum(v) => v.iter().map(evaluate).min().unwrap(),
        Expression::Maximum(v) => v.iter().map(evaluate).max().unwrap(),
        Expression::LiteralValue(v) => *v,
        Expression::GreaterThan(v) => (evaluate(&v[0]) > evaluate(&v[1])) as u64,
        Expression::LessThan(v) => (evaluate(&v[0]) < evaluate(&v[1])) as u64,
        Expression::EqualTo(v) => (evaluate(&v[0]) == evaluate(&v[1])) as u64,
        Expression::None => 0,
    }
}

fn decode_hex_packet(input: &str) -> String {
    input
        .chars()
        .map(|v| format!("{:04b}", u8::from_str_radix(&String::from(v), 16).unwrap()))
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
struct Packet {
    version: u8,
    data: Expression,
}

#[derive(Debug, Eq, PartialEq)]
enum Expression {
    Sum(Vec<Packet>),
    Product(Vec<Packet>),
    Minimum(Vec<Packet>),
    Maximum(Vec<Packet>),
    LiteralValue(u64),
    GreaterThan(Vec<Packet>),
    LessThan(Vec<Packet>),
    EqualTo(Vec<Packet>),
    None,
}

fn decode_bin_packet(buffer: &mut &[u8]) -> Packet {
    let version = decode_bin_number::<3>(buffer) as u8;
    let type_id = decode_bin_number::<3>(buffer) as u8;
    match type_id {
        4 => {
            let mut literal_value = String::new();
            loop {
                let literal_group = read_bits::<5>(buffer);
                literal_value += String::from_utf8_lossy(&literal_group[1..5]).as_ref();
                if literal_group[0] == b'0' {
                    break;
                }
            }
            Packet {
                version,
                data: Expression::LiteralValue(u64::from_str_radix(&literal_value, 2).unwrap()),
            }
        }
        _ => {
            let mut sub_packets = Vec::new();
            let length_type_id = read_bits::<1>(buffer);
            if length_type_id[0] == b'0' {
                let sub_packets_length = decode_bin_number::<15>(buffer) as usize;
                decode_sub_packets(&mut &buffer[0..sub_packets_length], &mut sub_packets);
                buffer.consume(sub_packets_length);
            } else if length_type_id[0] == b'1' {
                let sub_packets_number = decode_bin_number::<11>(buffer) as usize;
                decode_n_sub_packets(sub_packets_number, buffer, &mut sub_packets);
            }
            Packet {
                version,
                data: match type_id {
                    0 => Expression::Sum(sub_packets),
                    1 => Expression::Product(sub_packets),
                    2 => Expression::Minimum(sub_packets),
                    3 => Expression::Maximum(sub_packets),
                    5 => Expression::GreaterThan(sub_packets),
                    6 => Expression::LessThan(sub_packets),
                    7 => Expression::EqualTo(sub_packets),
                    _ => Expression::None,
                },
            }
        }
    }
}

fn decode_sub_packets(buffer: &mut &[u8], sub_packets: &mut Vec<Packet>) {
    while !buffer.is_empty() {
        sub_packets.push(decode_bin_packet(buffer));
    }
}

fn decode_n_sub_packets(n: usize, buffer: &mut &[u8], sub_packets: &mut Vec<Packet>) {
    for _ in 0..n {
        sub_packets.push(decode_bin_packet(buffer));
    }
}

fn read_bits<const BITS: usize>(buffer: &mut &[u8]) -> [u8; BITS] {
    let mut data = [0u8; BITS];
    buffer.read_exact(&mut data).unwrap();
    data
}

fn decode_bin_number<const BITS: usize>(buffer: &mut &[u8]) -> u16 {
    let number_data: [u8; BITS] = read_bits(buffer);
    u16::from_str_radix(String::from_utf8_lossy(&number_data).as_ref(), 2).unwrap()
}

#[test]
fn decode_hex_packet_test() {
    assert_eq!(decode_hex_packet("D2FE28"), "110100101111111000101000");
}

#[test]
fn decode_bin_literal_packet_test() {
    assert_eq!(
        decode_bin_packet(&mut b"110100101111111000101000".as_slice()),
        Packet {
            version: 6,
            data: Expression::LiteralValue(2021),
        }
    );
}

#[test]
fn decode_bin_operator_packet_with_limit_by_size_test() {
    assert_eq!(
        decode_bin_packet(
            &mut b"00111000000000000110111101000101001010010001001000000000".as_slice()
        ),
        Packet {
            version: 1,
            data: Expression::LessThan(vec![
                Packet {
                    version: 6,
                    data: Expression::LiteralValue(10),
                },
                Packet {
                    version: 2,
                    data: Expression::LiteralValue(20),
                }
            ])
        }
    );
}

#[test]
fn decode_bin_operator_packet_with_limit_by_number_test() {
    assert_eq!(
        decode_bin_packet(
            &mut b"11101110000000001101010000001100100000100011000001100000".as_slice()
        ),
        Packet {
            version: 7,
            data: Expression::Maximum(vec![
                Packet {
                    version: 2,
                    data: Expression::LiteralValue(1),
                },
                Packet {
                    version: 4,
                    data: Expression::LiteralValue(2),
                },
                Packet {
                    version: 1,
                    data: Expression::LiteralValue(3),
                }
            ])
        }
    );
}

#[test]
fn example_1_test() {
    let buffer = r#"C200B40A82
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 3);
}

#[test]
fn example_2_test() {
    let buffer = r#"04005AC33890
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 54);
}

#[test]
fn example_3_test() {
    let buffer = r#"880086C3E88112
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 7);
}

#[test]
fn example_4_test() {
    let buffer = r#"CE00C43D881120
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 9);
}

#[test]
fn example_5_test() {
    let buffer = r#"D8005AC2A8F0
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 1);
}

#[test]
fn example_6_test() {
    let buffer = r#"F600BC2D8F
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 0);
}

#[test]
fn example_7_test() {
    let buffer = r#"9C005AC2F8F0
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 0);
}

#[test]
fn example_8_test() {
    let buffer = r#"9C0141080250320F1802104A08
"#
    .as_bytes();
    assert_eq!(evaluate_packet(buffer), 1);
}
