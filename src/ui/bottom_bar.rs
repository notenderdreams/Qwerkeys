use egui::{Align, FontId, Layout, RichText, Ui};

use super::QwerkeysApp;

impl QwerkeysApp {
    pub(super) fn render_bottom_bar(&self, ui: &mut Ui) {
        let theme = self.theme_colors;
        let event_info = self.state.last_event_info.lock().unwrap().clone();

        ui.spacing_mut().item_spacing.y = 0.0;
        ui.spacing_mut().interact_size.y = 11.0;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("STATUS:")
                    .font(FontId::monospace(9.5))
                    .strong()
                    .color(theme.TEXT_MUTED),
            );

            ui.label(
                RichText::new(&event_info)
                    .font(FontId::monospace(10.5))
                    .strong()
                    .color(theme.TEXT_PRIMARY),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .font(FontId::monospace(9.0))
                        .strong()
                        .color(theme.TEXT_MUTED),
                );

                ui.separator();

                ui.label(
                    RichText::new("PORT: qwerkeys-input")
                        .font(FontId::monospace(9.0))
                        .color(theme.TEXT_MUTED),
                );
            });
        });
    }
}
