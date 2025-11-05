use memmap2::Mmap;
use std::{fs::File, io};
use zstd::decode_all;

use crate::readers::{TimsTofFileType, TimsTofPathError, TimsTofPathLike};

const HEADER_BYTES: usize = 8;

#[derive(Debug)]
pub struct TsfBlobReader {
    mmap: Mmap,
}

impl TsfBlobReader {
    pub fn new(path: impl TimsTofPathLike) -> Result<Self, TsfBlobReaderError> {
        let path = path.to_timstof_path()?;
        let tsf_bin = match path.file_type() {
            #[cfg(feature = "tsf")]
            TimsTofFileType::TSF => path.tsf_bin()?,
            _ => return Err(TsfBlobReaderError::WrongDataset),
        };
        let file = File::open(tsf_bin)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn read_chunk(
        &self,
        offset: usize,
        num_peaks: usize,
    ) -> Result<TsfSpectrumChunk, TsfBlobReaderError> {
        let header = self.read_header(offset)?;
        let compressed_start = offset + HEADER_BYTES;
        let compressed_end = compressed_start + header.compressed_len;
        let compressed = self
            .mmap
            .get(compressed_start..compressed_end)
            .ok_or(TsfBlobReaderError::CorruptData)?;
        let decompressed = decode_all(compressed)
            .map_err(TsfBlobReaderError::Decompression)?;
        // check that the number of peaks matches with the decompressed data length. 
        // Each peak uses 12 bytes
        let expected = num_peaks
            .checked_mul(12)
            .ok_or(TsfBlobReaderError::Overflow)?;
        if decompressed.len() < expected {
            return Err(TsfBlobReaderError::UnexpectedLength {
                expected,
                actual: decompressed.len(),
            });
        }
        let (mz_bytes, intensity_bytes) = decompressed.split_at(num_peaks * 8);
        let mz = mz_bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        let intensities = intensity_bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        Ok(TsfSpectrumChunk { mz, intensities })
    }

    fn read_header(&self, offset: usize) -> Result<TsfChunkHeader, TsfBlobReaderError> {
        let header_bytes = self
            .mmap
            .get(offset..offset + HEADER_BYTES)
            .ok_or(TsfBlobReaderError::CorruptData)?;
        let chunk_padded = u32::from_le_bytes(header_bytes[0..4].try_into().unwrap()) as usize;
        let compressed_len = u32::from_le_bytes(header_bytes[4..8].try_into().unwrap()) as usize;
        if chunk_padded < HEADER_BYTES || chunk_padded < compressed_len {
            return Err(TsfBlobReaderError::UnexpectedLength {
                expected: chunk_padded,
                actual: compressed_len,
            });
        }
        Ok(TsfChunkHeader {
            _chunk_padded: chunk_padded,
            compressed_len,
        })
    }
}

#[derive(Debug)]
pub struct TsfSpectrumChunk {
    pub mz: Vec<f64>,
    pub intensities: Vec<f32>,
}

#[derive(Debug)]
struct TsfChunkHeader {
    _chunk_padded: usize,
    compressed_len: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum TsfBlobReaderError {
    #[error("{0}")]
    TimsTofPathError(#[from] TimsTofPathError),
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Data is corrupt")]
    CorruptData,
    #[error("Unexpected length (expected at least {expected}, got {actual})")]
    UnexpectedLength { expected: usize, actual: usize },
    #[error("Integer overflow while computing peak payload size")]
    Overflow,
    #[error("Wrong dataset type for TSF reader")]
    WrongDataset,
    #[error("Decompression failed")]
    Decompression(std::io::Error),
}
