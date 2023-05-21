use egui::{Color32, TextStyle};
use egui_extras::{Column, TableBuilder, TableRow};

pub trait View {
    type Item;

    fn id(&self) -> egui::Id;

    fn display_item(&mut self, item: &Self::Item, row: TableRow<'_, '_>) -> egui::Response;

    fn build_table<'t>(&self, builder: TableBuilder<'t>) -> TableBuilder<'t> {
        builder.column(Column::remainder())
    }

    fn selected_color(&self) -> egui::Color32 {
        Color32::YELLOW
    }

    fn display_footer_hint(&mut self, ui: &mut egui::Ui, input: &str) {
        self.on_nothing_found(ui, input)
    }

    fn bg_fill(&self, style: &egui::Style) -> egui::Color32 {
        style.visuals.extreme_bg_color.gamma_multiply(0.9)
    }

    fn previous_entry_pressed(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
    }

    fn next_entry_pressed(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
    }

    fn top_entry(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::Home)
    }

    fn bottom_entry(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::End)
    }

    fn previous_entry_page(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::PageUp)
    }

    fn next_entry_page(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::PageDown)
    }

    fn accept_entry_pressed(&self, input: &mut egui::InputState) -> bool {
        input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
    }

    fn on_nothing_found(&mut self, ui: &mut egui::Ui, input: &str) {
        ui.scope(|ui| {
            let fid = TextStyle::Body.resolve(ui.style());
            ui.spacing_mut().item_spacing.x = ui.fonts(|f| f.glyph_width(&fid, ' '));
            ui.label("Nothing found for:");
            let resp = ui.monospace(input);
            ui.painter().line_segment(
                [resp.rect.left_bottom(), resp.rect.right_bottom()],
                (1.0, ui.visuals().warn_fg_color),
            )
        });
    }

    fn window_padding(&self, style: &egui::Style) -> f32 {
        let _ = style;
        40.0
    }
}
