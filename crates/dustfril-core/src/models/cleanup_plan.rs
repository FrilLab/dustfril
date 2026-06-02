use crate::models::CleanupCandidate;

#[derive(Debug, Default)]
pub struct CleanupPlan {
    pub candidates: Vec<CleanupCandidate>,

    pub reclaimable_size_bytes: u64,
}
