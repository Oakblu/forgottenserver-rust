use std::sync::Arc;

use criterion::{criterion_group, criterion_main, black_box, Criterion};
use forgottenserver_common::itemloader::ItemGroup;
use forgottenserver_items::container::Container;
use forgottenserver_items::item::Item;
use forgottenserver_items::items_registry::{ItemTypeData, ItemTypeKind};
use forgottenserver_items::registry::{ItemsRegistry, ItemType};

fn make_registry_item(server_id: u16) -> ItemType {
    ItemType {
        server_id,
        client_id: server_id + 1000,
        group: ItemGroup::Ground,
        flags: 0,
        speed: 0,
        weight: 10,
    }
}

fn make_item_type_data(id: u16) -> Arc<ItemTypeData> {
    Arc::new(ItemTypeData {
        id,
        client_id: id + 1000,
        group: ItemGroup::Ground,
        type_kind: ItemTypeKind::None,
        pickupable: true,
        moveable: true,
        ..Default::default()
    })
}

fn registry_build_100(c: &mut Criterion) {
    c.bench_function("registry_build_100", |b| {
        b.iter(|| {
            let mut r = ItemsRegistry::new();
            for id in 1u16..=100 {
                r.register(make_registry_item(black_box(id)));
            }
            black_box(r.len())
        });
    });
}

fn registry_build_1000(c: &mut Criterion) {
    c.bench_function("registry_build_1000", |b| {
        b.iter(|| {
            let mut r = ItemsRegistry::new();
            for id in 1u16..=1000 {
                r.register(make_registry_item(black_box(id)));
            }
            black_box(r.len())
        });
    });
}

fn registry_lookup_hit(c: &mut Criterion) {
    let mut r = ItemsRegistry::new();
    for id in 1u16..=1000 {
        r.register(make_registry_item(id));
    }

    c.bench_function("registry_lookup_hit", |b| {
        b.iter(|| {
            black_box(r.get(black_box(500u16)))
        });
    });
}

fn registry_lookup_miss(c: &mut Criterion) {
    let mut r = ItemsRegistry::new();
    for id in 1u16..=1000 {
        r.register(make_registry_item(id));
    }

    c.bench_function("registry_lookup_miss", |b| {
        b.iter(|| {
            black_box(r.get(black_box(9999u16)))
        });
    });
}

fn registry_lookup_sequential_100(c: &mut Criterion) {
    let mut r = ItemsRegistry::new();
    for id in 1u16..=1000 {
        r.register(make_registry_item(id));
    }

    c.bench_function("registry_lookup_sequential_100", |b| {
        b.iter(|| {
            for id in 1u16..=100 {
                black_box(r.get(black_box(id)));
            }
        });
    });
}

fn container_add_20_items(c: &mut Criterion) {
    let item_type = make_item_type_data(1);

    c.bench_function("container_add_20_items", |b| {
        b.iter(|| {
            let mut container = Container::new(black_box(1u16), 20);
            for _ in 0..20 {
                let item = Item::new(Arc::clone(&item_type), 1);
                let _ = container.add_item(black_box(item));
            }
            black_box(container.size())
        });
    });
}

criterion_group!(
    benches,
    registry_build_100,
    registry_build_1000,
    registry_lookup_hit,
    registry_lookup_miss,
    registry_lookup_sequential_100,
    container_add_20_items,
);
criterion_main!(benches);
