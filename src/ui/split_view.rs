use eframe::egui;

/// Split orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

/// A split pane with its state.
#[derive(Debug, Clone)]
pub struct SplitPane {
    /// Pane index.
    pub index: usize,
    /// Scroll offset.
    pub scroll_offset: f32,
    /// Cursor line.
    pub cursor_line: usize,
    /// Cursor column.
    pub cursor_col: usize,
    /// Whether this pane is active.
    pub active: bool,
}

impl SplitPane {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            scroll_offset: 0.0,
            cursor_line: 0,
            cursor_col: 0,
            active: index == 0,
        }
    }
}

/// Split view state.
#[derive(Debug, Clone)]
pub struct SplitView {
    /// Whether split view is enabled.
    pub enabled: bool,
    /// Split orientation.
    pub orientation: SplitOrientation,
    /// Split ratio (0.0 to 1.0).
    pub split_ratio: f32,
    /// Panes in the split.
    pub panes: Vec<SplitPane>,
    /// Active pane index.
    pub active_pane: usize,
}

impl SplitView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            orientation: SplitOrientation::Horizontal,
            split_ratio: 0.5,
            panes: vec![SplitPane::new(0)],
            active_pane: 0,
        }
    }

    /// Toggle split view.
    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }

    /// Enable split view.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.panes = vec![SplitPane::new(0), SplitPane::new(1)];
        self.active_pane = 0;
        self.panes[0].active = true;
    }

    /// Disable split view (return to single pane).
    pub fn disable(&mut self) {
        self.enabled = false;
        self.panes = vec![SplitPane::new(0)];
        self.active_pane = 0;
    }

    /// Set the active pane.
    pub fn set_active_pane(&mut self, index: usize) {
        if index < self.panes.len() {
            for pane in &mut self.panes {
                pane.active = false;
            }
            self.panes[index].active = true;
            self.active_pane = index;
        }
    }

    /// Get the active pane.
    pub fn active_pane(&self) -> &SplitPane {
        &self.panes[self.active_pane]
    }

    /// Get the active pane mutably.
    pub fn active_pane_mut(&mut self) -> &mut SplitPane {
        &mut self.panes[self.active_pane]
    }

    /// Toggle orientation.
    pub fn toggle_orientation(&mut self) {
        self.orientation = match self.orientation {
            SplitOrientation::Horizontal => SplitOrientation::Vertical,
            SplitOrientation::Vertical => SplitOrientation::Horizontal,
        };
    }

    /// Update split ratio (from drag).
    pub fn set_split_ratio(&mut self, ratio: f32) {
        self.split_ratio = ratio.clamp(0.1, 0.9);
    }

    /// Show the split view UI.
    pub fn show(&self, ui: &mut egui::Ui, render_pane: impl Fn(usize, &mut egui::Ui)) {
        if !self.enabled {
            render_pane(0, ui);
            return;
        }

        match self.orientation {
            SplitOrientation::Horizontal => {
                ui.horizontal(|ui| {
                    let total_width = ui.available_width();
                    let pane1_width = total_width * self.split_ratio;
                    let pane2_width = total_width * (1.0 - self.split_ratio);

                    ui.vertical(|ui| {
                        ui.set_min_width(pane1_width);
                        ui.set_max_width(pane1_width);
                        render_pane(0, ui);
                    });

                    // Splitter
                    let splitter_rect = ui.available_rect_before_wrap();
                    let _response = ui.allocate_rect(splitter_rect, egui::Sense::drag());

                    ui.vertical(|ui| {
                        ui.set_min_width(pane2_width);
                        ui.set_max_width(pane2_width);
                        render_pane(1, ui);
                    });
                });
            }
            SplitOrientation::Vertical => {
                ui.vertical(|ui| {
                    let total_height = ui.available_height();
                    let pane1_height = total_height * self.split_ratio;
                    let pane2_height = total_height * (1.0 - self.split_ratio);

                    ui.set_min_height(pane1_height);
                    ui.set_max_height(pane1_height);
                    render_pane(0, ui);

                    // Splitter
                    let splitter_rect = ui.available_rect_before_wrap();
                    ui.allocate_rect(splitter_rect, egui::Sense::drag());

                    ui.set_min_height(pane2_height);
                    ui.set_max_height(pane2_height);
                    render_pane(1, ui);
                });
            }
        }
    }
}

impl Default for SplitView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_view_toggle() {
        let mut sv = SplitView::new();
        assert!(!sv.enabled);
        sv.toggle();
        assert!(sv.enabled);
        assert_eq!(sv.panes.len(), 2);
        sv.toggle();
        assert!(!sv.enabled);
        assert_eq!(sv.panes.len(), 1);
    }

    #[test]
    fn test_active_pane() {
        let mut sv = SplitView::new();
        sv.enable();
        assert_eq!(sv.active_pane, 0);
        sv.set_active_pane(1);
        assert_eq!(sv.active_pane, 1);
        assert!(sv.panes[1].active);
    }

    #[test]
    fn test_split_ratio() {
        let mut sv = SplitView::new();
        sv.set_split_ratio(0.3);
        assert!((sv.split_ratio - 0.3).abs() < f32::EPSILON);
        sv.set_split_ratio(2.0); // Clamped
        assert!((sv.split_ratio - 0.9).abs() < f32::EPSILON);
    }
}
