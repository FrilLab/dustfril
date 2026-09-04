use super::CleanupRecommendation;

/// Default inactivity age used for cleanup recommendations.
pub const DEFAULT_CLEANUP_AGE_DAYS: u64 = 30;

/// The single policy used to classify analyzed artifacts for cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecommendationPolicy {
    cleanup_age_days: u64,
}

impl Default for RecommendationPolicy {
    fn default() -> Self {
        Self {
            cleanup_age_days: DEFAULT_CLEANUP_AGE_DAYS,
        }
    }
}

impl RecommendationPolicy {
    /// Creates a policy for a positive cleanup age.
    pub fn new(cleanup_age_days: u64) -> Option<Self> {
        (cleanup_age_days > 0).then_some(Self { cleanup_age_days })
    }

    /// Returns the configured age at which an artifact becomes reclaimable.
    pub const fn cleanup_age_days(self) -> u64 {
        self.cleanup_age_days
    }

    /// Returns the age at which an artifact needs review.
    ///
    /// This uses integer ceiling division so every positive cleanup age has a
    /// deterministic, non-overlapping Keep / NeedsReview / SafeToClean range.
    pub const fn review_age_days(self) -> u64 {
        self.cleanup_age_days / 2 + self.cleanup_age_days % 2
    }

    /// Classifies an artifact from its whole-day age.
    pub const fn recommendation(self, age_days: Option<u64>) -> CleanupRecommendation {
        let Some(age_days) = age_days else {
            return CleanupRecommendation::NeedsReview;
        };

        if age_days < self.review_age_days() {
            CleanupRecommendation::Keep
        } else if age_days < self.cleanup_age_days {
            CleanupRecommendation::NeedsReview
        } else {
            CleanupRecommendation::SafeToClean
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_thirty_days() {
        let policy = RecommendationPolicy::default();

        assert_eq!(policy.cleanup_age_days(), 30);
        assert_eq!(policy.review_age_days(), 15);
    }

    #[test]
    fn policy_uses_ceiling_half_for_review_boundary() {
        assert_eq!(RecommendationPolicy::new(7).unwrap().review_age_days(), 4);
        assert_eq!(RecommendationPolicy::new(60).unwrap().review_age_days(), 30);
    }

    #[test]
    fn policy_rejects_zero_days() {
        assert!(RecommendationPolicy::new(0).is_none());
    }

    #[test]
    fn policy_classifies_default_boundaries() {
        let policy = RecommendationPolicy::default();

        assert_eq!(policy.recommendation(Some(14)), CleanupRecommendation::Keep);
        assert_eq!(
            policy.recommendation(Some(15)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(29)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(30)),
            CleanupRecommendation::SafeToClean
        );
        assert_eq!(
            policy.recommendation(Some(31)),
            CleanupRecommendation::SafeToClean
        );
    }

    #[test]
    fn policy_classifies_sixty_day_boundaries() {
        let policy = RecommendationPolicy::new(60).unwrap();

        assert_eq!(policy.recommendation(Some(29)), CleanupRecommendation::Keep);
        assert_eq!(
            policy.recommendation(Some(30)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(59)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(60)),
            CleanupRecommendation::SafeToClean
        );
    }

    #[test]
    fn policy_classifies_seven_day_boundaries() {
        let policy = RecommendationPolicy::new(7).unwrap();

        assert_eq!(policy.recommendation(Some(3)), CleanupRecommendation::Keep);
        assert_eq!(
            policy.recommendation(Some(4)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(6)),
            CleanupRecommendation::NeedsReview
        );
        assert_eq!(
            policy.recommendation(Some(7)),
            CleanupRecommendation::SafeToClean
        );
    }

    #[test]
    fn unknown_age_needs_review() {
        assert_eq!(
            RecommendationPolicy::default().recommendation(None),
            CleanupRecommendation::NeedsReview
        );
    }
}
