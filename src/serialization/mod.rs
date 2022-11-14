use bevy::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Resource)]
struct MapSelection(PathBuf);

fn load(map: Res<MapSelection>) {}
