use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forgottenserver_common::position::Position;
use forgottenserver_game::combat::{Combat, CombatArea, CombatType};

fn bench_get_block_type(c: &mut Criterion) {
    let types = [
        CombatType::None,
        CombatType::Physical,
        CombatType::Energy,
        CombatType::Earth,
        CombatType::Fire,
        CombatType::Undefined,
        CombatType::LifeDrain,
        CombatType::ManaDrain,
        CombatType::Healing,
        CombatType::Drown,
        CombatType::Ice,
        CombatType::Holy,
        CombatType::Death,
    ];
    let immunity_flags: u16 = 0b0000_0101_0101_0101;

    c.bench_function("combat_get_block_type", |b| {
        b.iter(|| {
            for &ct in &types {
                black_box(Combat::get_block_type(black_box(ct), black_box(immunity_flags)));
            }
        })
    });
}

fn bench_clamp_damage(c: &mut Criterion) {
    c.bench_function("combat_clamp_damage", |b| {
        b.iter(|| {
            black_box(Combat::clamp_damage_to_health(black_box(500), black_box(300)));
            black_box(Combat::clamp_damage_to_health(black_box(100), black_box(300)));
            black_box(Combat::clamp_damage_to_health(black_box(0), black_box(300)));
            black_box(Combat::clamp_damage_to_health(black_box(-10), black_box(300)));
        })
    });
}

fn bench_apply_critical_hit(c: &mut Criterion) {
    c.bench_function("combat_apply_critical_hit", |b| {
        b.iter(|| {
            black_box(Combat::apply_critical_hit(black_box(-200), black_box(50)));
            black_box(Combat::apply_critical_hit(black_box(-500), black_box(100)));
            black_box(Combat::apply_critical_hit(black_box(150), black_box(25)));
        })
    });
}

fn bench_compute_leech(c: &mut Criterion) {
    c.bench_function("combat_compute_leech", |b| {
        b.iter(|| {
            black_box(Combat::compute_leech_amount(black_box(400), black_box(2500)));
            black_box(Combat::compute_leech_amount(black_box(1000), black_box(500)));
            black_box(Combat::compute_leech_amount(black_box(0), black_box(9999)));
        })
    });
}

fn bench_get_tiles_in_range_small(c: &mut Criterion) {
    let center = Position::new(100, 100, 7);

    c.bench_function("combat_get_tiles_in_range_small", |b| {
        b.iter(|| {
            black_box(CombatArea::get_tiles_in_range(black_box(center), black_box(3), black_box(3)));
        })
    });
}

fn bench_get_tiles_in_range_large(c: &mut Criterion) {
    let center = Position::new(100, 100, 7);

    c.bench_function("combat_get_tiles_in_range_large", |b| {
        b.iter(|| {
            black_box(CombatArea::get_tiles_in_range(black_box(center), black_box(7), black_box(7)));
        })
    });
}

fn bench_is_protected(c: &mut Criterion) {
    let cases: &[(u32, u32, u32)] = &[
        (1, 1, 8),
        (10, 5, 8),
        (100, 100, 8),
        (7, 20, 8),
        (50, 50, 100),
    ];

    c.bench_function("combat_is_protected", |b| {
        b.iter(|| {
            for &(atk, tgt, prot) in cases {
                black_box(Combat::is_protected(
                    black_box(atk),
                    black_box(tgt),
                    black_box(prot),
                ));
            }
        })
    });
}

fn bench_formula_damage(c: &mut Criterion) {
    c.bench_function("combat_formula_damage", |b| {
        b.iter(|| {
            black_box(Combat::apply_formula_damage(
                black_box(100),
                black_box(50),
                black_box(50),
                black_box(200),
                black_box(137),
            ));
            black_box(Combat::apply_formula_damage(
                black_box(1),
                black_box(1),
                black_box(10),
                black_box(10),
                black_box(0),
            ));
            black_box(Combat::apply_formula_damage(
                black_box(500),
                black_box(80),
                black_box(1),
                black_box(999),
                black_box(42),
            ));
        })
    });
}

criterion_group!(
    benches,
    bench_get_block_type,
    bench_clamp_damage,
    bench_apply_critical_hit,
    bench_compute_leech,
    bench_get_tiles_in_range_small,
    bench_get_tiles_in_range_large,
    bench_is_protected,
    bench_formula_damage,
);
criterion_main!(benches);
