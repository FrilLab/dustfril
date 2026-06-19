use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupRecommendation {
    Keep,
    Review,
    SafeToClean,
}

impl fmt::Display for CleanupRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanupRecommendation::Keep => {
                write!(f, "Keep")
            }

            CleanupRecommendation::Review => {
                write!(f, "Review")
            }

            CleanupRecommendation::SafeToClean => {
                write!(f, "Safe To Clean")
            }
        }
    }
}
