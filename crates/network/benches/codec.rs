use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_common::xtea;

fn output_message_build_small(c: &mut Criterion) {
    c.bench_function("output_message_build_small", |b| {
        b.iter(|| {
            let mut msg = OutputMessage::new();
            msg.add_u8(black_box(0x01));
            msg.add_u8(black_box(0x02));
            msg.add_u8(black_box(0x03));
            msg.add_u8(black_box(0x04));
            msg.add_u16(black_box(0x1234));
            msg.add_u16(black_box(0x5678));
            msg.add_string(black_box("hello world"));
            msg.write_message_length();
            black_box(msg.get_message_length())
        });
    });
}

fn output_message_build_large(c: &mut Criterion) {
    c.bench_function("output_message_build_large", |b| {
        let strings = [
            "first string payload",
            "second string payload",
            "third string payload",
            "fourth string payload",
            "fifth string payload",
        ];
        b.iter(|| {
            let mut msg = OutputMessage::new();
            for i in 0u8..50 {
                msg.add_u8(black_box(i));
            }
            for i in 0u16..20 {
                msg.add_u16(black_box(i * 100 + 1));
            }
            for s in &strings {
                msg.add_string(black_box(s));
            }
            msg.write_message_length();
            black_box(msg.get_message_length())
        });
    });
}

fn output_message_write_length(c: &mut Criterion) {
    let mut msg = OutputMessage::new();
    for i in 0u8..50 {
        msg.add_u8(i);
    }
    msg.add_u32(0xDEAD_BEEF);
    msg.add_string("benchmark payload string");

    c.bench_function("output_message_write_length", |b| {
        b.iter(|| {
            let mut m = OutputMessage::new();
            m.add_u32(black_box(0xDEAD_BEEF));
            m.add_u32(black_box(0xCAFE_BABE));
            m.add_string(black_box("test string"));
            m.write_message_length();
            black_box(m.get_output_buffer().len())
        });
    });
}

fn network_message_read_u8s(c: &mut Criterion) {
    let mut prepared = NetworkMessage::new();
    for i in 0u8..10 {
        prepared.add_u8(i + 1);
    }

    c.bench_function("network_message_read_u8s", |b| {
        b.iter(|| {
            let mut msg = prepared.clone();
            msg.set_buffer_position(0);
            let mut sum: u32 = 0;
            for _ in 0..10 {
                sum = sum.wrapping_add(black_box(msg.get_u8()) as u32);
            }
            black_box(sum)
        });
    });
}

fn network_message_read_mixed(c: &mut Criterion) {
    let mut prepared = NetworkMessage::new();
    prepared.add_u8(0xAB);
    prepared.add_u16(0x1234);
    prepared.add_u32(0xDEAD_BEEF);
    prepared.add_string("packet data");

    c.bench_function("network_message_read_mixed", |b| {
        b.iter(|| {
            let mut msg = prepared.clone();
            msg.set_buffer_position(0);
            let a = black_box(msg.get_u8());
            let b = black_box(msg.get_u16());
            let d = black_box(msg.get_u32());
            let s = black_box(msg.get_string(0));
            black_box((a, b, d, s.len()))
        });
    });
}

fn xtea_roundtrip_packet(c: &mut Criterion) {
    let key = xtea::Key([0xDEAD_BEEF, 0xCAFE_BABE, 0x1234_5678, 0xABCD_EF01]);
    let round_keys = xtea::expand_key(&key);

    let mut msg = OutputMessage::new();
    msg.add_u8(0x14);
    msg.add_u32(0x0000_0001);
    msg.add_string("player_name");
    msg.add_u16(100);
    msg.add_u16(200);
    msg.add_u8(7);
    msg.write_message_length();

    let payload: Vec<u8> = {
        let buf = msg.get_output_buffer();
        let len = buf.len();
        let padded_len = len.div_ceil(8) * 8;
        let mut v = vec![0u8; padded_len];
        v[..len].copy_from_slice(buf);
        v
    };

    c.bench_function("xtea_roundtrip_packet", |b| {
        b.iter(|| {
            let mut data = black_box(payload.clone());
            xtea::encrypt(&mut data, &round_keys);
            xtea::decrypt(&mut data, &round_keys);
            black_box(data[0])
        });
    });
}

criterion_group!(
    benches,
    output_message_build_small,
    output_message_build_large,
    output_message_write_length,
    network_message_read_u8s,
    network_message_read_mixed,
    xtea_roundtrip_packet,
);
criterion_main!(benches);
