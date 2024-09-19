/// A collection of user-modifiable settings. Should be expanded as new features are added.
pub struct Settings {
    pub line_length: usize,
}

#[allow(dead_code)]
pub fn default_settings() -> Settings {
    Settings { line_length: 100 }
}
