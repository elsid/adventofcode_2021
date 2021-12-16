use std::io::{BufRead, Read};

fn main() {
    println!("{}", sum_packet_versions(std::io::stdin().lock()));
}

fn sum_packet_versions(buffer: impl BufRead) -> u64 {
    buffer
        .lines()
        .map(Result::unwrap)
        .map(|v| decode_hex_packet(&v))
        .map(|v| decode_bin_packet(&mut v.as_bytes()))
        .map(|v| get_packet_version_sum(&v))
        .sum()
}

fn get_packet_version_sum(packet: &Packet) -> u64 {
    packet.version as u64
        + match &packet.data {
            PacketData::SubPackets(v) => v.iter().map(get_packet_version_sum).sum(),
            _ => 0,
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
    data: PacketData,
}

#[derive(Debug, Eq, PartialEq)]
enum PacketData {
    SubPackets(Vec<Packet>),
    LiteralValue(u64),
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
                data: PacketData::LiteralValue(u64::from_str_radix(&literal_value, 2).unwrap()),
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
                data: PacketData::SubPackets(sub_packets),
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
            data: PacketData::LiteralValue(2021),
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
            data: PacketData::SubPackets(vec![
                Packet {
                    version: 6,
                    data: PacketData::LiteralValue(10),
                },
                Packet {
                    version: 2,
                    data: PacketData::LiteralValue(20),
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
            data: PacketData::SubPackets(vec![
                Packet {
                    version: 2,
                    data: PacketData::LiteralValue(1),
                },
                Packet {
                    version: 4,
                    data: PacketData::LiteralValue(2),
                },
                Packet {
                    version: 1,
                    data: PacketData::LiteralValue(3),
                }
            ])
        }
    );
}

#[test]
fn example_1_test() {
    let buffer = r#"8A004A801A8002F478
"#
    .as_bytes();
    assert_eq!(sum_packet_versions(buffer), 16);
}

#[test]
fn example_2_test() {
    let buffer = r#"C0015000016115A2E0802F182340
"#
    .as_bytes();
    assert_eq!(sum_packet_versions(buffer), 23);
}

#[test]
fn example_3_test() {
    let buffer = r#"A0016C880162017C3686B18A3D4780
"#
    .as_bytes();
    assert_eq!(sum_packet_versions(buffer), 31);
}
