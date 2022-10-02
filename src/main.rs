#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use utils::*;

use bevy::prelude::*;
use derive_more::From;

mod editor;
mod hit;
mod sheet;
mod timing;
mod utils;

enum GameState {
    Browse,
    Edit,
    Play,
}

fn main() {
    /*App::new()
    .insert_resource(Msaa { samples: 4 })
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_startup_system(setup_system)
    .run();*/
}

/*fn setup_system(mut commands: Commands) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(200.0),
        ..shapes::RegularPolygon::default()
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::CYAN),
            outline_mode: StrokeMode::new(Color::BLACK, 10.0),
        },
        Transform::default(),
    ));
}*/
