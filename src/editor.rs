use bevy::prelude::*;

struct SelectionItems {
    item: usize,
    sub_itme: Option<usize>,
}

struct Selection {
    entity: Entity,
    items: Option<SelectionItems>,
}
