use crate::{
    domain_converters::{ConvertableDomain, Tof2MzConverter},
    io::readers::{
        file_readers::{
            sql_reader::{
                metadata::SqlMetadata,
                tsf_frames::SqlTsfFrame,
                ReadableSqlTable,
                ReadableSqlHashMap,
                SqlReader,
                SqlReaderError,
            },
            tsf_blob_reader::{TsfBlobReader, TsfBlobReaderError},
        },
        MetadataReader, MetadataReaderError,
    },
    ms_data::Spectrum,
    readers::TimsTofPathLike,
};

use super::{SpectrumReaderConfig, SpectrumReaderError, SpectrumReaderTrait};

#[derive(Debug)]
pub struct TSFSpectrumReader {
    frames: Vec<TsfFrame>,
    blob_reader: TsfBlobReader,
    mz_converter: Tof2MzConverter,
    _config: SpectrumReaderConfig,
}

impl TSFSpectrumReader {
    pub fn new(
        path: impl TimsTofPathLike,
        config: SpectrumReaderConfig,
    ) -> Result<Self, TSFSpectrumReaderError> {
        let blob_reader = TsfBlobReader::new(&path)?;
        let sql_reader = SqlReader::open(&path)?;
        let metadata_map = SqlMetadata::from_sql_reader(&sql_reader)?;
        let has_line_spectra = metadata_map
            .get("HasLineSpectra")
            .map(|value| value.trim() == "1")
            .unwrap_or(false);
        if !has_line_spectra {
            return Err(TSFSpectrumReaderError::UnsupportedDataset);
        }
        let frames = SqlTsfFrame::from_sql_reader(&sql_reader)?;
        let metadata = MetadataReader::new(&path)?;
        let frames = frames
            .into_iter()
            .map(|frame| TsfFrame {
                frame_id: frame.id,
                offset: frame.binary_offset,
                num_peaks: frame.peak_count,
                _rt_seconds: frame.rt,
            })
            .collect();
        Ok(Self {
            frames,
            blob_reader,
            mz_converter: metadata.mz_converter,
            _config: config,
        })
    }

    fn read_spectrum(&self, index: usize) -> Result<Spectrum, TSFSpectrumReaderError> {
        let frame = self
            .frames
            .get(index)
            .ok_or(TSFSpectrumReaderError::IndexOutOfBounds)?;
        let chunk = self
            .blob_reader
            .read_chunk(frame.offset, frame.num_peaks)?;
        let mz_values = chunk
            .tof
            .into_iter()
            .map(|tof| self.mz_converter.convert(tof))
            .collect();
        let intensities = chunk
            .intensities
            .into_iter()
            .map(|value| value as f64)
            .collect();
        let spectrum = Spectrum {
            mz_values,
            intensities,
            precursor: None,
            index: frame.frame_id,
            collision_energy: 0.0,
            isolation_mz: 0.0,
            isolation_width: 0.0,
        };
        Ok(spectrum)
    }
}

impl SpectrumReaderTrait for TSFSpectrumReader {
    fn get(&self, index: usize) -> Result<Spectrum, SpectrumReaderError> {
        Ok(self.read_spectrum(index)?)
    }

    fn len(&self) -> usize {
        self.frames.len()
    }

    fn calibrate(&mut self) {
        // todo?
    }
}

#[derive(Debug)]
struct TsfFrame {
    frame_id: usize,
    offset: usize,
    num_peaks: usize,
    _rt_seconds: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TSFSpectrumReaderError {
    #[error("{0}")]
    SqlReaderError(#[from] SqlReaderError),
    #[error("{0}")]
    MetadataReaderError(#[from] MetadataReaderError),
    #[error("{0}")]
    TsfBlobReaderError(#[from] TsfBlobReaderError),
    #[error("Spectrum index out of bounds")]
    IndexOutOfBounds,
    #[error("TSF dataset does not contain centroided line spectra")]
    UnsupportedDataset,
}
