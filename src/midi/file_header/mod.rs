pub mod division;

#[derive(Debug, PartialEq)]
pub struct MidiFileHeader {
    pub format: MidiFileFormat,
    pub num_tracks: u16,
    pub division: u16,
}

#[derive(Debug, PartialEq)]
pub enum MidiFileFormat {
    SingleTrack,
    MultiTrack,
    MultiSong,
}

impl MidiFileHeader {
    /// Create a new MidiFileHeader struct
    ///
    /// ### Arguments
    /// * `format` The format of the MIDI file
    /// * `num_tracks` The number of tracks in the MIDI file
    /// * `division` The division of the MIDI file
    ///
    /// ### Returns
    /// A new MidiFileHeader struct
    pub fn new(format: MidiFileFormat, num_tracks: u16, division: u16) -> MidiFileHeader {
        MidiFileHeader {
            format,
            num_tracks,
            division,
        }
    }

    /// Parse a MIDI file header from a ByteReader
    ///
    /// ### Arguments
    /// * `reader` The ByteReader to read from
    ///
    /// ### Returns
    /// A MidiFileHeader struct
    pub fn new_from_reader(
        reader: &mut crate::bytereader::ByteReader,
    ) -> crate::errors::MidiParseResult<MidiFileHeader> {
        // Validate the header
        let header = reader.get_next_bytes(4)?;
        if header != b"MThd" {
            return Err(crate::errors::MidiParseError::InvalidHeader(
                "Invalid header".to_string(),
            ));
        }

        // Validate the header length
        let header_length = reader.get_next_u32()?;
        if header_length != 6 {
            return Err(crate::errors::MidiParseError::InvalidDataBounds(
                "Invalid header length".to_string(),
            ));
        }

        // Get the next following bytes after the length are 16-bit words
        // First word is the format, get it and validate it
        let format = reader.get_next_u16()?;
        let format = MidiFileHeader::format_from_u16(format)?;

        // The next word is the number of tracks
        // If the format is SingleTrack, this should be 1
        let num_tracks = reader.get_next_u16()?;
        if format == MidiFileFormat::SingleTrack && num_tracks != 1 {
            return Err(crate::errors::MidiParseError::InvalidNumOfTracks(
                "Single track MIDI files should only have one track chunk".to_string(),
            ));
        }

        // The next word is the division

        Ok(MidiFileHeader {
            format,
            num_tracks,
            division: 0,
        })
    }

    /// Convert the bytes to the MidiFileFormat enum
    ///
    /// ### Arguments
    /// * `format` The format as a u16
    ///
    /// ### Returns
    /// A MidiFileFormat enum
    fn format_from_u16(format: u16) -> crate::errors::MidiParseResult<MidiFileFormat> {
        match format {
            0 => Ok(MidiFileFormat::SingleTrack),
            1 => Ok(MidiFileFormat::MultiTrack),
            2 => Ok(MidiFileFormat::MultiSong),
            _ => Err(crate::errors::MidiParseError::InvalidFileFormat(
                "Invalid file format".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let header = MidiFileHeader::new(MidiFileFormat::SingleTrack, 0, 0);
        assert_eq!(header.format, MidiFileFormat::SingleTrack);
        assert_eq!(header.num_tracks, 0);
        assert_eq!(header.division, 0);
    }

    #[test]
    fn test_new_from_reader() {
        let data = b"MThd\x00\x00\x00\x06\x00\x01\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let header = MidiFileHeader::new_from_reader(&mut reader).unwrap();
    }

    #[test]
    fn test_new_from_reader_invalid_header() {
        let data = b"Invalid\x00\x00\x00\x06\x00\x01\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let result = MidiFileHeader::new_from_reader(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_from_reader_invalid_header_length() {
        let data = b"MThd\x00\x00\x00\x07\x00\x01\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let result = MidiFileHeader::new_from_reader(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_from_u16() {
        let format = MidiFileHeader::format_from_u16(0).unwrap();
        assert_eq!(format, MidiFileFormat::SingleTrack);

        let format = MidiFileHeader::format_from_u16(1).unwrap();
        assert_eq!(format, MidiFileFormat::MultiTrack);

        let format = MidiFileHeader::format_from_u16(2).unwrap();
        assert_eq!(format, MidiFileFormat::MultiSong);

        let format = MidiFileHeader::format_from_u16(3);
        assert!(format.is_err());
    }

    #[test]
    fn test_new_from_reader_valid_format() {
        let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let header = MidiFileHeader::new_from_reader(&mut reader).unwrap();
        assert_eq!(header.format, MidiFileFormat::SingleTrack);
    }

    #[test]
    fn test_new_from_reader_invalid_format() {
        let data = b"MThd\x00\x00\x00\x06\x00\x03\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let result = MidiFileHeader::new_from_reader(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_from_reader_valid_number_tracks() {
        let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let header = MidiFileHeader::new_from_reader(&mut reader).unwrap();
        assert_eq!(header.num_tracks, 1);
    }

    #[test]
    fn test_new_from_reader_invalid_number_tracks() {
        let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x02\x00\x80";
        let mut reader = crate::bytereader::ByteReader::new(data);
        let result = MidiFileHeader::new_from_reader(&mut reader);
        assert!(result.is_err());
    }
}
