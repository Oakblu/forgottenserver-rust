use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forgottenserver_entity::creature::{ConditionEntry, Creature, Direction};

fn bench_condition_entry_new(c: &mut Criterion) {
    c.bench_function("condition_entry_new", |b| {
        b.iter(|| ConditionEntry::new(black_box(1 << 3), black_box(5000)));
    });
}

fn bench_direction_is_diagonal(c: &mut Criterion) {
    let dirs = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::NorthEast,
        Direction::SouthEast,
        Direction::SouthWest,
        Direction::NorthWest,
    ];

    c.bench_function("direction_is_diagonal", |b| {
        b.iter(|| {
            let mut result = false;
            for &d in &dirs {
                result ^= black_box(d).is_diagonal();
            }
            result
        });
    });
}

fn bench_creature_add_10_conditions(c: &mut Criterion) {
    c.bench_function("creature_add_10_conditions", |b| {
        b.iter(|| {
            let mut creature = Creature::new(black_box(1), "Rat");
            for i in 0..10_i32 {
                creature.add_condition(black_box(1 << i), black_box(5000 + i * 100));
            }
            creature
        });
    });
}

fn bench_creature_tick_conditions(c: &mut Criterion) {
    let mut setup = Creature::new(1, "Rat");
    for i in 0..10_i32 {
        setup.add_condition(1 << i, 100_000);
    }

    c.bench_function("creature_tick_conditions", |b| {
        b.iter_batched(
            || {
                let mut creature = Creature::new(1, "Rat");
                for i in 0..10_i32 {
                    creature.add_condition(1 << i, 100_000);
                }
                creature
            },
            |mut creature| {
                creature.tick_conditions(black_box(1000));
                creature
            },
            criterion::BatchSize::SmallInput,
        );
    });

    let _ = setup;
}

fn bench_creature_get_speed(c: &mut Criterion) {
    let creature = Creature::new(1, "Rat");

    c.bench_function("creature_get_speed", |b| {
        b.iter(|| black_box(&creature).get_speed());
    });
}

fn bench_creature_get_walk_delay(c: &mut Criterion) {
    let creature = Creature::new(1, "Rat");

    c.bench_function("creature_get_walk_delay_straight", |b| {
        b.iter(|| black_box(&creature).get_walk_delay(black_box(false)));
    });

    c.bench_function("creature_get_walk_delay_diagonal", |b| {
        b.iter(|| black_box(&creature).get_walk_delay(black_box(true)));
    });
}

fn bench_creature_has_condition(c: &mut Criterion) {
    let mut creature = Creature::new(1, "Rat");
    for i in 0..10_i32 {
        creature.add_condition(1 << i, 100_000);
    }

    c.bench_function("creature_has_condition_hit", |b| {
        b.iter(|| creature.has_condition(black_box(1 << 9)));
    });

    c.bench_function("creature_has_condition_miss", |b| {
        b.iter(|| creature.has_condition(black_box(1 << 15)));
    });
}

criterion_group!(
    benches,
    bench_condition_entry_new,
    bench_direction_is_diagonal,
    bench_creature_add_10_conditions,
    bench_creature_tick_conditions,
    bench_creature_get_speed,
    bench_creature_get_walk_delay,
    bench_creature_has_condition,
);
criterion_main!(benches);
