#[derive(Debug, PartialEq)]
pub enum MidiParseError {
    EndOfData(String),
    InvalidDataBounds(String),
    InvalidHeader(String),
    InvalidFileFormat(String),
    InvalidNumOfTracks(String),
}

pub type MidiParseResult<T> = Result<T, MidiParseError>;

impl std::error::Error for MidiParseError {}

impl std::fmt::Display for MidiParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiParseError::EndOfData(msg) => {
                write!(f, "End of data: {}: (Code: {})", msg, self.code())
            }
            MidiParseError::InvalidDataBounds(msg) => write!(f, "Invalid data bounds: {}", msg),
            MidiParseError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            MidiParseError::InvalidFileFormat(msg) => write!(f, "Invalid file format: {}", msg),
            MidiParseError::InvalidNumOfTracks(msg) => {
                write!(f, "Invalid number of tracks: {}", msg)
            }
        }
    }
}

impl MidiParseError {
    pub fn code(&self) -> u32 {
        match self {
            MidiParseError::EndOfData(_) => 0x00,
            MidiParseError::InvalidDataBounds(_) => 0x01,
            MidiParseError::InvalidHeader(_) => 0x02,
            MidiParseError::InvalidFileFormat(_) => 0x03,
            MidiParseError::InvalidNumOfTracks(_) => 0x04,
        }
    }
}
