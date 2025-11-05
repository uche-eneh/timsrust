use crate::{ms_data::Precursor, readers::TimsTofPathLike};

use super::PrecursorReaderTrait;

// Spectrum builder still expects to hand back a PrecursorReaderTrait, 
// so the TSF implementation returns an empty reader

#[derive(Debug, Default)]
pub struct TSFPrecursorReader;

impl TSFPrecursorReader {
    pub fn new(_path: impl TimsTofPathLike) -> Self {
        // TSF exports do not carry precursor metadata, so we return an empty reader.
        Self
    }
}

impl PrecursorReaderTrait for TSFPrecursorReader {
    fn get(&self, _index: usize) -> Option<Precursor> {
        None
    }

    fn len(&self) -> usize {
        0
    }
}
