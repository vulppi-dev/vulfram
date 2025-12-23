use std::ops::Range;

use super::types::STREAM_COUNT;

#[derive(Debug, Default)]
pub struct BindCache {
    pub last_index: Option<(usize, Range<u64>)>,
    pub last_vertex: [Option<(usize, Range<u64>)>; STREAM_COUNT],
}

impl BindCache {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
