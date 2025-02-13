use crate::bytereader::ByteReader;

pub mod file_header;

pub struct MidiFile {
    pub placeholder: u8,
}

impl MidiFile {
    /// Create a new MidiFile struct from a file
    ///
    /// ### Arguments
    /// * `file_path` The path to the file to read
    ///
    /// ### Returns
    /// A new MidiFile struct
    pub fn new_from_file(file_path: &str) -> MidiFile {
        MidiFile { placeholder: 0 }
    }

    /// Create a new MidiFile struct from a byte array
    ///
    /// ### Arguments
    /// * `data` The byte array to read
    ///
    /// ### Returns
    /// A new MidiFile struct
    pub fn new_from_data(data: &[u8]) -> MidiFile {
        // First, create a new ByteReader
        let mut reader = ByteReader::new(data);

        MidiFile { placeholder: 0 }
    }
}
