#[cfg(feature = "minitdf")]
use super::minitdf::MiniTDFSpectrumReaderError;
#[cfg(feature = "tdf")]
use super::tdf::TDFSpectrumReaderError;
#[cfg(feature = "tsf")]
use super::tsf::TSFSpectrumReaderError;

#[derive(Debug, thiserror::Error)]
pub enum SpectrumReaderError {
    #[cfg(feature = "minitdf")]
    #[error("{0}")]
    MiniTDFSpectrumReaderError(#[from] MiniTDFSpectrumReaderError),
    #[cfg(feature = "tdf")]
    #[error("{0}")]
    TDFSpectrumReaderError(#[from] TDFSpectrumReaderError),
    #[cfg(feature = "tsf")]
    #[error("{0}")]
    TSFSpectrumReaderError(#[from] TSFSpectrumReaderError),
    #[error("No path provided")]
    NoPath,
}
