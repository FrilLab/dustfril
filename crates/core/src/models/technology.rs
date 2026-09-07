use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Structured technology context for a discovered project.
///
/// The display label is deliberately derived in Core so callers do not need
/// to infer project identity from an artifact directory name or reproduce
/// detector rules in a presentation layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTechnology {
    /// One or more languages supported by the project.
    pub languages: Vec<String>,
    /// Runtime or ecosystem associated with the project, when applicable.
    pub runtime: Option<String>,
    /// Build system or package/build tool, when applicable.
    pub build_system: Option<String>,
    /// Compact canonical label intended for user-facing surfaces.
    pub display_label: String,
    /// Bounded metadata evidence used to establish this identity.
    pub evidence: Vec<TechnologyEvidence>,
}

/// One metadata item that contributed to technology detection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TechnologyEvidence {
    pub path: PathBuf,
    pub detail: String,
}

impl ProjectTechnology {
    pub fn new(
        languages: Vec<String>,
        runtime: Option<String>,
        build_system: Option<String>,
        evidence: Vec<TechnologyEvidence>,
    ) -> Self {
        let display_label = display_label(&languages, runtime.as_deref(), build_system.as_deref());

        Self {
            languages,
            runtime,
            build_system,
            display_label,
            evidence,
        }
    }

    /// Creates a useful compatibility identity for manually constructed
    /// artifacts that do not pass through a detector.
    pub fn for_ecosystem(ecosystem: super::Ecosystem) -> Self {
        let (language, runtime, build_system) = match ecosystem {
            super::Ecosystem::Rust => ("Rust", None, Some("Cargo")),
            super::Ecosystem::Node => ("JavaScript", Some("Node.js"), None),
            super::Ecosystem::Java => ("Java", None, None),
            super::Ecosystem::CMake => ("C++", None, Some("CMake")),
            super::Ecosystem::DotNet => ("C#", Some(".NET"), None),
            super::Ecosystem::Python => ("Python", None, None),
            super::Ecosystem::Swift => ("Swift", None, None),
            super::Ecosystem::Dart => ("Dart", None, None),
            super::Ecosystem::Flutter => ("Dart", Some("Dart"), Some("Flutter")),
            super::Ecosystem::Kotlin => ("Kotlin", None, Some("Gradle")),
            super::Ecosystem::Php => ("PHP", None, None),
            super::Ecosystem::Elixir => ("Elixir", None, None),
            super::Ecosystem::Zig => ("Zig", None, None),
            super::Ecosystem::Go => ("Go", None, None),
            super::Ecosystem::Ruby => ("Ruby", None, None),
        };

        Self::new(
            vec![language.to_owned()],
            runtime.map(str::to_owned),
            build_system.map(str::to_owned),
            Vec::new(),
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.languages.is_empty()
            && self.runtime.is_none()
            && self.build_system.is_none()
            && self.display_label.is_empty()
            && self.evidence.is_empty()
    }
}

fn display_label(
    languages: &[String],
    runtime: Option<&str>,
    build_system: Option<&str>,
) -> String {
    let language = if languages.is_empty() {
        "Unknown".to_owned()
    } else {
        languages.join("/")
    };

    match (runtime, build_system) {
        (Some("Dart"), Some("Flutter")) => "Flutter · Dart".to_owned(),
        (Some(runtime), _) if !matches!(runtime, "Node.js" | ".NET") => {
            format!("{language} · {runtime}")
        }
        (Some(runtime), _) if language != "JavaScript" && language != "TypeScript" => {
            format!("{language} · {runtime}")
        }
        (Some(runtime), _) => format!("{language} · {runtime}"),
        (_, Some(build_system))
            if !matches!(build_system, "Cargo" | "SPM" | "Pub" | "Mix" | "Zig") =>
        {
            format!("{language} · {build_system}")
        }
        _ => language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_labels_keep_language_and_build_context_distinct() {
        assert_eq!(
            ProjectTechnology::new(
                vec!["C++".to_owned()],
                None,
                Some("CMake".to_owned()),
                Vec::new(),
            )
            .display_label,
            "C++ · CMake"
        );
        assert_eq!(
            ProjectTechnology::new(
                vec!["TypeScript".to_owned()],
                Some("Node.js".to_owned()),
                None,
                Vec::new(),
            )
            .display_label,
            "TypeScript · Node.js"
        );
        assert_eq!(
            ProjectTechnology::new(
                vec!["Dart".to_owned()],
                Some("Dart".to_owned()),
                Some("Flutter".to_owned()),
                Vec::new(),
            )
            .display_label,
            "Flutter · Dart"
        );
    }
}
