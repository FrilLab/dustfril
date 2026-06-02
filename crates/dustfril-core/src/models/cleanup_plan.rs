use crate::models::CleanupCandidate;

#[derive(Debug, Default)]
pub struct CleanupPlan {
    pub candidates: Vec<CleanupCandidate>,
}

impl CleanupPlan {
    pub fn reclaimable_size_bytes(&self) -> u64 {
        self.candidates
            .iter()
            .map(|candidate| candidate.size_bytes)
            .sum()
    }
}
