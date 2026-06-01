use crate::models::CleanupRecommendation;

pub fn recommend_cleanup(age_days: Option<u64>) -> CleanupRecommendation {
    let Some(days) = age_days else {
        return CleanupRecommendation::Review;
    };

    match days {
        0..=30 => CleanupRecommendation::Keep,

        31..=90 => CleanupRecommendation::Review,

        _ => CleanupRecommendation::SafeToClean,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_when_recent() {
        assert_eq!(recommend_cleanup(Some(10)), CleanupRecommendation::Keep);
    }

    #[test]
    fn review_when_middle_age() {
        assert_eq!(recommend_cleanup(Some(60)), CleanupRecommendation::Review);
    }

    #[test]
    fn safe_to_clean_when_old() {
        assert_eq!(
            recommend_cleanup(Some(180)),
            CleanupRecommendation::SafeToClean
        );
    }
}
