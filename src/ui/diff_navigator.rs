use eframe::egui;

use crate::app::RustpadApp;

/// Diff navigator state.
#[derive(Debug, Default)]
pub struct DiffNavigator {
    pub current_hunk_index: usize,
    pub total_hunks: usize,
}

impl DiffNavigator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_hunk(&mut self) {
        if self.current_hunk_index + 1 < self.total_hunks {
            self.current_hunk_index += 1;
        }
    }

    pub fn prev_hunk(&mut self) {
        if self.current_hunk_index > 0 {
            self.current_hunk_index -= 1;
        }
    }

    pub fn goto_hunk(&mut self, index: usize) {
        if index < self.total_hunks {
            self.current_hunk_index = index;
        }
    }
}

/// Legacy hook; diff navigation lives in detached compare viewports.
#[allow(dead_code)]
pub fn show(_app: &mut RustpadApp, _ctx: &egui::Context) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_navigator() {
        let mut nav = DiffNavigator::new();
        nav.total_hunks = 5;

        nav.next_hunk();
        assert_eq!(nav.current_hunk_index, 1);

        nav.prev_hunk();
        assert_eq!(nav.current_hunk_index, 0);
    }
}
