use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{ArtifactAnalysis, Ecosystem};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupPlan {
    pub candidates: Vec<CleanupCandidate>,
}

impl CleanupPlan {
    /// Returns the total number of bytes that can be reclaimed by this plan.
    pub fn reclaimable_size_bytes(&self) -> u64 {
        self.candidates
            .iter()
            .map(|candidate| candidate.size_bytes)
            .sum()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_paths: Vec<PathBuf>,
    pub failed_paths: Vec<PathBuf>,
    pub freed_size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupRecommendation {
    Keep,
    NeedsReview,
    SafeToClean,
}

impl fmt::Display for CleanupRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keep => write!(f, "Keep"),

            Self::NeedsReview => write!(f, "NeedsReview"),

            Self::SafeToClean => write!(f, "SafeToClean"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub path: PathBuf,
    pub ecosystem: Ecosystem,
    pub size_bytes: u64,
    pub age_days: Option<u64>,
}

impl From<ArtifactAnalysis> for CleanupCandidate {
    fn from(analysis: ArtifactAnalysis) -> Self {
        Self {
            path: analysis.artifact.path,
            ecosystem: analysis.artifact.ecosystem,
            size_bytes: analysis.size_bytes,
            age_days: analysis.age_days,
        }
    }
}
