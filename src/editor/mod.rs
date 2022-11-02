use bevy::prelude::*;

enum Selection {
    Single(Entity),
    Multi(Vec<Entity>),
    SingleItem(Entity, usize),
    MultiItem(Entity, Vec<usize>),
}

struct ClipBoard(Selection);

#[derive(Component)]
struct Seeker(f64);
