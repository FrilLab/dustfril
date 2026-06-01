use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Target,
    CargoRegistry,
    CargoGit,
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArtifactType::Target => {
                write!(f, "TARGET")
            }
            ArtifactType::CargoRegistry => {
                write!(f, "CARGO REGISTRY")
            }
            ArtifactType::CargoGit => {
                write!(f, "CARGO GIT")
            }
        }
    }
}
