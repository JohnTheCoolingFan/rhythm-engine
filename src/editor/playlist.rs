use super::*;
use crate::harmonizer::arranger::*;
use bevy::prelude::*;
use bevy_egui::*;

fn sheets<T: Default + Component>(
    mut egui_context: ResMut<EguiContext>,
    instances: SequenceSheets<T>,
    focus: Res<Focus>,
) {
    let Focus::Channel(channel) = *focus else { return };
}
