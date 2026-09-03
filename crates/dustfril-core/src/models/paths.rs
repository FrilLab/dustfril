use std::path::Path;

/// Returns true only when `ancestor` is the same path as, or a path-component
/// ancestor of, `descendant`.
///
/// `Path::starts_with` compares components rather than raw strings, so paths
/// such as `node_modules` and `node_modules-cache` are not confused.
pub(crate) fn path_contains(ancestor: &Path, descendant: &Path) -> bool {
    ancestor == descendant || descendant.starts_with(ancestor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn containment_is_path_component_aware() {
        assert!(path_contains(
            Path::new("/workspace/project/node_modules"),
            Path::new("/workspace/project/node_modules/foo/node_modules"),
        ));
        assert!(!path_contains(
            Path::new("/workspace/project/node_modules"),
            Path::new("/workspace/project/node_modules-cache"),
        ));
    }
}
