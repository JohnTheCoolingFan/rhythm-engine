use bevy_egui::{egui::Ui, *};

fn activations(ui: &mut Ui) {
    egui::TopBottomPanel::top("Activations")
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.label("Activations");
            egui::ScrollArea::both().show(ui, |ui| {});
        });
}

fn tracks(ui: &mut Ui) {
    egui::TopBottomPanel::bottom("Tracks")
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.label("Tracks");
            egui::ScrollArea::both().show(ui, |ui| {});
        });
}

fn routings(ui: &mut Ui) {
    egui::SidePanel::right("Routings")
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.label("Routings");
            egui::ScrollArea::both().show(ui, |ui| {});
        });
}

pub fn playlist_panel(mut contexts: EguiContexts) {
    egui::TopBottomPanel::bottom("Playlist")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            routings(ui);
            activations(ui);
            tracks(ui);
        });
}
