#[derive(Default)]
pub struct Search {
    pub(super) buffer: String,
    pub(super) state: State,
}

pub(crate) enum State {
    Selected {
        index: usize,
    },
    Scores {
        scores: Vec<fuzzy::Score>,
        index: usize,
    },
    NoMatch,
}

impl Default for State {
    fn default() -> Self {
        Self::Selected { index: 0 }
    }
}

#[derive(Default)]
pub enum Action {
    Accept {
        index: usize,
    },
    HasInput,
    #[default]
    Nothing,
}
