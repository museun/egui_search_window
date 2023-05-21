use egui::{
    epaint::Shadow, vec2, Color32, Frame, LayerId, Margin, Order, Rounding, Sense, Stroke,
    TextEdit, TextStyle, Ui,
};
use egui_extras::TableBuilder;

use super::{
    context::{Action, Search, State},
    view::View,
};

pub struct Window<'a, T> {
    items: &'a mut [T],
    state: &'a mut Search,
}

impl<'a, T> Window<'a, T>
where
    T: fuzzy::SearchItem + 'a,
{
    pub fn new(items: &'a mut [T], state: &'a mut Search) -> Self {
        Self { items, state }
    }

    pub fn display(mut self, ui: &mut egui::Ui, mut view: impl View<Item = T>) -> Action {
        let input_state = InputState::load(ui.ctx(), &mut view);

        Frame::none()
            .shadow(Shadow::small_dark())
            .fill(view.bg_fill(ui.style()))
            .outer_margin(Margin::symmetric(view.window_padding(ui.style()), 4.0))
            .show(ui, move |ui| {
                self.display_edit_box(ui);

                let buffer_empty = self.state.buffer.trim().is_empty();
                let has_items = !self.items.is_empty();
                if !has_items && !buffer_empty {
                    view.on_nothing_found(ui, self.state.buffer.trim());
                    return Action::HasInput;
                }

                if has_items {
                    match input_state.as_kind() {
                        InputKind::Down => self.handle_select_down(),
                        InputKind::Up => self.handle_select_up(),
                        InputKind::End => self.handle_select_list_down(),
                        InputKind::Home => self.handle_select_list_up(),
                        InputKind::Enter if !input_state.ctrl => {
                            if let Action::Accept { index } = self.handle_select_accept() {
                                return Action::Accept { index };
                            }
                        }
                        InputKind::Enter => return Action::HasInput,
                        _ => {}
                    }
                }

                let add_contents = |ui: &mut egui::Ui| match &self.state.state {
                    State::NoMatch if !buffer_empty => {
                        view.on_nothing_found(ui, self.state.buffer.trim());
                        Action::HasInput
                    }

                    State::NoMatch => {
                        view.on_nothing_found(ui, self.state.buffer.trim());
                        Action::Nothing
                    }

                    State::Selected { index } => DisplayItems {
                        items: self.items,
                        buffer: self.state.buffer.trim(),
                        range: (0..self.items.len()).collect(),
                        selected: *index,
                    }
                    .display(ui, &mut view),

                    State::Scores { scores, index } => DisplayItems {
                        items: self.items,
                        buffer: self.state.buffer.trim(),
                        range: scores.iter().map(|&s| s.index).collect(),
                        selected: *index,
                    }
                    .display(ui, &mut view),
                };

                Frame::none()
                    .inner_margin(Margin {
                        left: 12.0,
                        right: 12.0,
                        top: 0.0,
                        bottom: 4.0,
                    })
                    .show(ui, add_contents)
                    .inner
            })
            .inner
    }

    fn display_edit_box(&mut self, ui: &mut egui::Ui) {
        Frame::none()
            .stroke(Stroke::new(0.2, Color32::DARK_GRAY))
            .outer_margin(Margin {
                left: 4.0,
                right: 4.0,
                top: 2.0,
                bottom: 4.0,
            })
            .show(ui, |ui| {
                let h = ui.text_style_height(&TextStyle::Body);
                let edit = TextEdit::singleline(&mut self.state.buffer)
                    .frame(false)
                    .lock_focus(true)
                    .font(TextStyle::Body);

                let resp = ui.add_sized(vec2(ui.available_width(), h), edit);
                resp.request_focus();

                if resp.lost_focus() {
                    resp.request_focus();
                }

                // TODO backspace on an empty input shouldn't change the state
                if resp.changed() {
                    let scores = fuzzy::search(&self.state.buffer, self.items);
                    self.state.state = if scores.is_empty() {
                        State::NoMatch
                    } else {
                        State::Scores { scores, index: 0 }
                    }
                }
            });
    }
}

impl<'a, T> Window<'a, T> {
    fn handle_select_down(&mut self) {
        let Some((index, len)) = self.get_extents() else { return };
        *index = (*index + 1) % len
    }

    fn handle_select_up(&mut self) {
        let Some((index, len)) = self.get_extents() else { return };
        *index = index.checked_sub(1).unwrap_or(len - 1);
    }

    fn handle_select_list_down(&mut self) {
        let Some((index, len)) = self.get_extents() else { return };
        *index = len.saturating_sub(1);
    }

    fn handle_select_list_up(&mut self) {
        let Some((index, _len)) = self.get_extents() else { return };
        *index = 0;
    }

    fn handle_select_accept(&mut self) -> Action {
        let index = match &self.state.state {
            State::Selected { index } => *index,
            State::Scores { scores, index } => scores[*index].index,
            State::NoMatch => return Action::Nothing,
        };
        Action::Accept { index }
    }

    fn get_extents(&mut self) -> Option<(&mut usize, usize)> {
        let result = match &mut self.state.state {
            State::Selected { index } => (index, self.items.len()),
            State::Scores { scores, index } => (index, scores.len()),
            State::NoMatch => return None,
        };
        Some(result)
    }
}

#[derive(Default)]
enum InputKind {
    Down,
    Up,
    End,
    Home,
    Enter,
    #[default]
    None,
}

#[derive(Default, Copy, Clone)]
struct InputState {
    down: bool,
    up: bool,
    end: bool,
    home: bool,
    enter: bool,
    ctrl: bool,
}

impl InputState {
    fn load<T>(ctx: &egui::Context, view: &mut impl View<Item = T>) -> Self {
        ctx.input_mut(|i| Self {
            down: view.next_entry_pressed(i),
            up: view.previous_entry_pressed(i),
            end: view.bottom_entry(i),
            home: view.top_entry(i),
            enter: view.accept_entry_pressed(i),
            ctrl: i.modifiers.command_only(),
        })
    }

    fn as_kind(&self) -> InputKind {
        [
            (self.down, InputKind::Down),
            (self.up, InputKind::Up),
            (self.end, InputKind::End),
            (self.home, InputKind::Home),
            (self.enter, InputKind::Enter),
        ]
        .into_iter()
        .find_map(|(p, f)| p.then_some(f))
        .unwrap_or_default()
    }
}

struct DisplayItems<'a, 'b, T> {
    items: &'a [T],
    buffer: &'b str,
    range: Vec<usize>,
    selected: usize,
}

impl<'a, 'b, T> DisplayItems<'a, 'b, T>
where
    T: fuzzy::SearchItem + 'a,
{
    fn display(self, ui: &mut egui::Ui, view: &mut impl View<Item = T>) -> Action {
        let table_ui = Ui::new(
            ui.ctx().clone(),
            ui.layer_id(),
            view.id().with(self.buffer),
            ui.max_rect(),
            ui.clip_rect(),
        );

        let row_height = ui.text_style_height(&TextStyle::Body);
        let mut action = None;

        let rect = ui.available_rect_before_wrap();

        view.build_table(TableBuilder::new(ui)).body(|body| {
            body.rows(row_height, self.range.len(), |raw_index, row| {
                let index = self.range[raw_index];
                let mut resp = view.display_item(&self.items[index], row);
                resp.rect.extend_with_x(rect.right());

                let resp = table_ui.interact(resp.rect, resp.id, Sense::click());

                if resp.clicked() {
                    action.replace(Action::Accept { index });
                    return;
                }

                let gamma = match (resp.hovered(), self.selected == raw_index) {
                    (true, false) => 0.1,
                    (.., true) => 0.2,
                    _ => return,
                };

                table_ui
                    .ctx()
                    .layer_painter(LayerId::new(
                        Order::Foreground,
                        table_ui.id().with("search-view").with("hover").with(index),
                    ))
                    .rect_filled(
                        resp.rect,
                        Rounding::none(),
                        view.selected_color().gamma_multiply(gamma),
                    );
            });
        });

        if !self.buffer.is_empty() {
            view.display_footer_hint(ui, self.buffer);
        }

        action.unwrap_or_default()
    }
}
