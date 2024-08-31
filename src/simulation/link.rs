
#[derive(Debug, Clone)]
pub struct Link {
    pub(crate) from_idx: usize,
    pub(crate) to_idx: usize,
    pub(crate) min_length: f32,
    pub(crate) max_length: f32,
    pub(crate) stiffness: f32,
    pub(crate) damping: f32,
}
impl Link {
    pub fn new(from_idx: usize, to_idx: usize) -> Self {
        Self {
            from_idx,
            to_idx,
            min_length: 0.0,
            max_length: f32::MAX,
            stiffness: 1.0,
            damping: 1.0,
        }
    }

    pub fn min_length(mut self, val: f32) -> Self {
        self.min_length = val;
        self
    }
    pub fn max_length(mut self, val: f32) -> Self {
        self.max_length = val;
        self
    }
    /// Sets the stiffness of the Link
    /// If the damping is `0.0`, the stiffness wont have any effect
    pub fn stiffness(mut self, val: f32) -> Self {
        self.stiffness = val;
        self
    }
    /// Sets the damping of the link.
    /// A damping of `0.0` will make the link completely stiff, a damping of `1.0` will make it completely elastic
    pub fn damping(mut self, val: f32) -> Self {
        self.damping = val;
        self
    }
}