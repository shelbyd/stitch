pub trait TreeSpace: Sync {
    type Candidate;

    fn next_candidates(&self, candidate: Self::Candidate) -> impl IntoIterator<Item = Self::Candidate>;

    fn each_candidate(&self, candidate: Self::Candidate);
}
