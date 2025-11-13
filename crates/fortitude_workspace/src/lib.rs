pub mod configuration;
pub mod options;
pub mod options_base;
pub mod resolver;

#[cfg(test)]
mod tests {
    use std::path::Path;

    pub(crate) fn test_resource_path(path: impl AsRef<Path>) -> std::path::PathBuf {
        Path::new("../fortitude_linter/resources/test/").join(path)
    }
}
