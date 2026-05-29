use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forgottenserver_common::position::Position;
use forgottenserver_map::pathfinder::Pathfinder;
use forgottenserver_map::spectators::Spectators;
use forgottenserver_map::tile::{flags, Tile};

fn tile_add_remove_creature(c: &mut Criterion) {
    c.bench_function("tile_add_remove_creature", |b| {
        let mut tile = Tile::new(100, 100, 7);
        b.iter(|| {
            tile.add_creature(black_box(42));
            black_box(tile.remove_creature(black_box(42)));
        });
    });
}

fn tile_flag_check(c: &mut Criterion) {
    c.bench_function("tile_flag_check", |b| {
        let mut tile = Tile::new(100, 100, 7);
        tile.set_flag(flags::BLOCKSOLID);
        b.iter(|| {
            black_box(tile.has_flag(black_box(flags::BLOCKSOLID)));
        });
    });
}

fn tile_add_10_creatures(c: &mut Criterion) {
    c.bench_function("tile_add_10_creatures", |b| {
        b.iter(|| {
            let mut tile = Tile::new(100, 100, 7);
            for id in 0..10u32 {
                tile.add_creature(black_box(id));
            }
            black_box(tile.get_creature_count())
        });
    });
}

fn spectators_add_100(c: &mut Criterion) {
    c.bench_function("spectators_add_100", |b| {
        b.iter(|| {
            let mut spectators = Spectators::new();
            for id in 0..50u32 {
                spectators.add_creature(black_box(id), black_box(true));
            }
            for id in 50..100u32 {
                spectators.add_creature(black_box(id), black_box(false));
            }
            black_box(spectators.count())
        });
    });
}

fn spectators_filter_range(c: &mut Criterion) {
    let mut spectators = Spectators::new();
    for id in 0..50u32 {
        spectators.add_creature(id, true);
    }
    for id in 50..100u32 {
        spectators.add_creature(id, false);
    }
    let positions: Vec<Position> = (0..100u32)
        .map(|id| {
            let x = 100u16 + (id % 10) as u16;
            let y = 100u16 + (id / 10) as u16;
            Position::new(x, y, 7)
        })
        .collect();
    let center = Position::new(105, 105, 7);

    c.bench_function("spectators_filter_range", |b| {
        b.iter(|| {
            let result = spectators.filter_by_range(black_box(&center), black_box(5), |id| {
                positions.get(id as usize).copied()
            });
            black_box(result)
        });
    });
}

fn pathfinder_short_path(c: &mut Criterion) {
    let pf = Pathfinder::new();
    let from = Position::new(100, 100, 7);
    let to = Position::new(105, 103, 7);

    c.bench_function("pathfinder_short_path", |b| {
        b.iter(|| black_box(pf.find_path(black_box(from), black_box(to))));
    });
}

fn pathfinder_already_there(c: &mut Criterion) {
    let pf = Pathfinder::new();
    let pos = Position::new(100, 100, 7);

    c.bench_function("pathfinder_already_there", |b| {
        b.iter(|| black_box(pf.find_path(black_box(pos), black_box(pos))));
    });
}

criterion_group!(
    spatial_benches,
    tile_add_remove_creature,
    tile_flag_check,
    tile_add_10_creatures,
    spectators_add_100,
    spectators_filter_range,
    pathfinder_short_path,
    pathfinder_already_there,
);
criterion_main!(spatial_benches);
