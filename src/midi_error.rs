use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader,
    InvalidNumOfTracks,
    InvalidFileFormat,
    InvalidFPS,
    InvalidEventBytes(String),
    NotImplemented(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidHeader => write!(f, "Invalid MIDI header"),
            ParseError::InvalidNumOfTracks => write!(f, "Invalid number of tracks. There must be at least one"),
            ParseError::InvalidFileFormat => write!(f, "Invalid file format. The file format must be 0, 1, or 2"),
            ParseError::InvalidFPS => write!(f, "Invalid FPS value. It must be -24, -25, -29, or -30"),
            ParseError::InvalidEventBytes(ref e) => write!(f, "{}", e),
            ParseError::NotImplemented(ref e) => write!(f, "Not Implemented: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum EventError {
    InvalidEvent,
    InvalidKeySignature(String),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::InvalidEvent => write!(f, "Invalid event"),
            EventError::InvalidKeySignature(ref e) => write!(f, "Invalid Key Signature: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum MidiError {
    FromUtf8Error(std::string::FromUtf8Error),
    IoError(std::io::Error),
    FileError(usize),
    ParseError(ParseError),
    EventError(EventError),
}

// Implement Error for MidiError
impl std::error::Error for MidiError {}

impl std::fmt::Display for MidiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MidiError::FromUtf8Error(ref e) => write!(f, "Error Converting to UTF-8: {}", e),
            MidiError::FileError(ref e) => write!(f, "File error: {}", e),
            MidiError::IoError(ref e) => write!(f, "I/O error: {}", e),
            MidiError::ParseError(ref e) => write!(f, "Parse error: {}. Possible file corruption?", e),
            MidiError::EventError(ref e) => write!(f, "Event error: {}. Possible file corruption?", e),
        }
    }
}

impl From<std::io::Error> for MidiError {
    fn from(value: std::io::Error) -> Self {
        MidiError::IoError(value)
    }
}

impl From<usize> for MidiError {
    fn from(value: usize) -> Self {
        MidiError::FileError(value)
    }
}

impl From<std::string::FromUtf8Error> for MidiError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        MidiError::FromUtf8Error(value)
    }
}

impl From<ParseError> for MidiError {
    fn from(value: ParseError) -> Self {
        MidiError::ParseError(value)
    }
}

impl From<EventError> for MidiError {
    fn from(value: EventError) -> Self {
        MidiError::EventError(value)
    }
}