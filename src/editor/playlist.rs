use super::*;
use crate::harmonizer::arranger::*;
use bevy::prelude::*;
use bevy_egui::*;

fn sheet_clusters<T: Default + Component>(
    contexts: EguiContexts,
    instances: SequenceSheets<T>,
    focus: Res<Focus>,
) {
    let Focus::Channel(channel) = *focus else { return };
}
