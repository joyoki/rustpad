/// Minimap state.
#[derive(Debug)]
pub struct Minimap {
    pub enabled: bool,
    pub width: f32,
    pub line_height: f32,
    pub scroll_offset: f32,
}

impl Minimap {
    pub fn new() -> Self {
        Self {
            enabled: true,
            width: 100.0,
            line_height: 2.0,
            scroll_offset: 0.0,
        }
    }

    /// Toggle minimap visibility.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Default for Minimap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_default() {
        let minimap = Minimap::default();
        assert!(minimap.enabled);
        assert_eq!(minimap.width, 100.0);
    }

    #[test]
    fn test_minimap_toggle() {
        let mut minimap = Minimap::new();
        assert!(minimap.enabled);
        minimap.toggle();
        assert!(!minimap.enabled);
    }
}
