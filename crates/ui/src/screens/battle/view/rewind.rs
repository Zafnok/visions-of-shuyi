//! What the rewind screen shows (ticket 0307, ADR-0054): the past actions,
//! newest first, the charges left and the question before a rewind. Built
//! by [`RewindScreen::view`](super::super::rewind::RewindScreen::view).

/// The rewind screen's list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewindView {
    /// Its title.
    pub title: String,
    /// The charges left (`2 of 3 charges left`), or that none are.
    pub charges: String,
    /// Whether no charge is left.
    pub spent: bool,
    /// The past actions, newest first, one line each.
    pub entries: Vec<String>,
    /// Which entry is highlighted.
    pub focus: usize,
    /// What is said when there is nothing to go back to; `None` when there
    /// is.
    pub nothing: Option<String>,
    /// The question before a rewind, while it is asked.
    pub prompt: Option<String>,
}

#[cfg(test)]
impl RewindView {
    /// Every line of text the screen shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = vec![self.title.clone(), self.charges.clone()];
        texts.extend(self.entries.iter().cloned());
        texts.extend(self.nothing.clone());
        texts.extend(self.prompt.clone());
        texts
    }
}
