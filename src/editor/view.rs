/// Viewport state for the editor.
#[derive(Debug, Clone)]
pub struct EditorView {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub view_width: f32,
    pub view_height: f32,
    pub first_visible_line: usize,
    pub last_visible_line: usize,
    pub scroll_buffer_lines: usize,
    pub target_scroll_y: Option<f32>,
    pub scroll_speed: f32,
}

impl EditorView {
    pub fn new(line_height: f32, char_width: f32) -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            line_height,
            char_width,
            view_width: 800.0,
            view_height: 600.0,
            first_visible_line: 0,
            last_visible_line: 0,
            scroll_buffer_lines: 5,
            target_scroll_y: None,
            scroll_speed: 8.0,
        }
    }

    pub fn update_dimensions(&mut self, width: f32, height: f32) {
        self.view_width = width;
        self.view_height = height;
    }

    pub fn update_visible_lines(&mut self, total_lines: usize) {
        if self.line_height <= 0.0 {
            return;
        }

        let first = (self.scroll_y / self.line_height) as usize;
        let visible_count = (self.view_height / self.line_height).ceil() as usize;

        self.first_visible_line = first.saturating_sub(self.scroll_buffer_lines);
        self.last_visible_line = (first + visible_count + self.scroll_buffer_lines)
            .min(total_lines.saturating_sub(1));
    }

    pub fn visible_line_count(&self) -> usize {
        self.last_visible_line.saturating_sub(self.first_visible_line) + 1
    }

    pub fn scroll_to_line(&mut self, line: usize) {
        let target_y = line as f32 * self.line_height;
        self.target_scroll_y = Some(target_y);
    }

    pub fn update_smooth_scroll(&mut self) -> bool {
        if let Some(target) = self.target_scroll_y {
            let diff = target - self.scroll_y;
            if diff.abs() < 1.0 {
                self.scroll_y = target;
                self.target_scroll_y = None;
                return true;
            }
            self.scroll_y += diff / self.scroll_speed;
            return true;
        }
        false
    }

    pub fn scroll_by(&mut self, delta_y: f32, total_lines: usize) {
        let max_scroll = (total_lines as f32 * self.line_height - self.view_height).max(0.0);
        self.scroll_y = (self.scroll_y - delta_y).clamp(0.0, max_scroll);
        self.target_scroll_y = None;
    }

    pub fn line_y_position(&self, line: usize) -> f32 {
        line as f32 * self.line_height - self.scroll_y
    }

    pub fn col_x_position(&self, col: usize) -> f32 {
        col as f32 * self.char_width - self.scroll_x
    }

    pub fn line_from_y(&self, y: f32) -> usize {
        ((y + self.scroll_y) / self.line_height) as usize
    }

    pub fn col_from_x(&self, x: f32) -> usize {
        ((x + self.scroll_x) / self.char_width) as usize
    }

    /// Generate minimap data for visible lines.
    pub fn generate_minimap_data(&self, lines: &[String], scale: f32) -> Vec<MinimapLine> {
        let mut result = Vec::new();
        for line_idx in self.first_visible_line..=self.last_visible_line {
            if line_idx >= lines.len() {
                break;
            }
            let line_text = &lines[line_idx];
            result.push(MinimapLine {
                line_index: line_idx,
                content: line_text.clone(),
                y_offset: line_idx as f32 * scale,
                width: line_text.len() as f32 * scale,
            });
        }
        result
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new(20.0, 8.0)
    }
}

/// A single line in the minimap.
#[derive(Debug, Clone)]
pub struct MinimapLine {
    pub line_index: usize,
    pub content: String,
    pub y_offset: f32,
    pub width: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_lines() {
        let mut view = EditorView::new(20.0, 8.0);
        view.update_dimensions(800.0, 400.0);
        view.update_visible_lines(100);
        assert_eq!(view.first_visible_line, 0);
        assert!(view.last_visible_line > 0);
    }

    #[test]
    fn test_scroll_to_line() {
        let mut view = EditorView::new(20.0, 8.0);
        view.scroll_to_line(50);
        assert!(view.target_scroll_y.is_some());
    }

    #[test]
    fn test_scroll_by() {
        let mut view = EditorView::new(20.0, 8.0);
        view.update_dimensions(800.0, 400.0);
        view.scroll_by(-100.0, 1000);
        assert!(view.scroll_y > 0.0);
    }

    #[test]
    fn test_line_from_y() {
        let view = EditorView::new(20.0, 8.0);
        assert_eq!(view.line_from_y(0.0), 0);
        assert_eq!(view.line_from_y(40.0), 2);
    }

    #[test]
    fn test_minimap_data() {
        let mut view = EditorView::new(20.0, 8.0);
        view.update_dimensions(800.0, 400.0);
        let lines = vec!["hello".to_string(), "world".to_string(), "foo".to_string()];
        view.update_visible_lines(3);
        let data = view.generate_minimap_data(&lines, 2.0);
        assert!(!data.is_empty());
    }
}
