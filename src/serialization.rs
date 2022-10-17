use bevy::prelude::*;
use std::path::{Path, PathBuf};

struct MapSelection(PathBuf);

fn load(map: Res<MapSelection>) {}
