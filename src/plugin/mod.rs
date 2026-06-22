use serde::{Deserialize, Serialize};

/// Plugin API trait - defines the interface for editor plugins.
/// Plugins can hook into editor events and transform text.
pub trait PluginApi {
    /// Called when a file is opened.
    fn on_open(&mut self, _file_path: &str, _content: &str) -> PluginResult {
        PluginResult::default()
    }

    /// Called when a file is saved.
    fn on_save(&mut self, _file_path: &str, _content: &str) -> PluginResult {
        PluginResult::default()
    }

    /// Called when a key is pressed.
    fn on_key_press(&mut self, _key: &str, _modifiers: &[String]) -> PluginResult {
        PluginResult::default()
    }

    /// Transform text content (e.g., formatting, linting fixes).
    fn transform_text(&self, content: &str) -> String {
        content.to_string()
    }

    /// Get plugin name.
    fn name(&self) -> &str;

    /// Get plugin description.
    fn description(&self) -> &str;

    /// Get plugin version.
    fn version(&self) -> &str {
        "0.1.0"
    }
}

/// Result from a plugin operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    /// Whether the plugin handled the event.
    pub handled: bool,
    /// Optional modified content.
    pub modified_content: Option<String>,
    /// Optional message to display.
    pub message: Option<String>,
}

impl Default for PluginResult {
    fn default() -> Self {
        Self {
            handled: false,
            modified_content: None,
            message: None,
        }
    }
}

/// Plugin manager - manages all loaded plugins.
pub struct PluginManager {
    plugins: Vec<Box<dyn PluginApi>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: Box<dyn PluginApi>) {
        self.plugins.push(plugin);
    }

    /// Notify all plugins of file open.
    pub fn on_open(&mut self, file_path: &str, content: &str) {
        for plugin in &mut self.plugins {
            let _ = plugin.on_open(file_path, content);
        }
    }

    /// Notify all plugins of file save.
    pub fn on_save(&mut self, file_path: &str, content: &str) {
        for plugin in &mut self.plugins {
            let _ = plugin.on_save(file_path, content);
        }
    }

    /// Notify all plugins of key press.
    pub fn on_key_press(&mut self, key: &str, modifiers: &[String]) {
        for plugin in &mut self.plugins {
            let _ = plugin.on_key_press(key, modifiers);
        }
    }

    /// Transform text through all plugins.
    pub fn transform_text(&self, content: &str) -> String {
        let mut result = content.to_string();
        for plugin in &self.plugins {
            result = plugin.transform_text(&result);
        }
        result
    }

    /// Get list of registered plugin names.
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Get plugin count.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
    }

    impl PluginApi for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A test plugin"
        }

        fn transform_text(&self, content: &str) -> String {
            content.to_uppercase()
        }
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);

        manager.register(Box::new(TestPlugin {
            name: "test".to_string(),
        }));
        assert_eq!(manager.plugin_count(), 1);
        assert_eq!(manager.plugin_names()[0], "test");
    }

    #[test]
    fn test_transform_text() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin {
            name: "test".to_string(),
        }));

        let result = manager.transform_text("hello");
        assert_eq!(result, "HELLO");
    }
}
