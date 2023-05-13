use crate::midi_error::{MidiError, ParseError};

#[cfg(feature = "json")]
use serde::Serialize;

const FILE_FORMAT_BYTES: std::ops::Range<usize> = 8..10;
const NUM_OF_TRACKS_BYTES: std::ops::Range<usize> = 10..12;
const TIME_DIVISION_BYTES: std::ops::Range<usize> = 12..14;

const HEADER_CHUNK_MTHD_BYTES: u32 = u32::from_be_bytes([0x4D, 0x54, 0x68, 0x64]);

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum FileFormat {
    SINGLE_TRACK,
    MULTI_TRACK,
    MULTI_SONG,
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum TimeDivision {
    PulsesPerQuarterNote(u16),
    SMPTE(u8, u8),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct MetaData {
    pub num_of_tracks: u16,
    pub file_format: FileFormat,
    pub time_division: TimeDivision,
}

impl MetaData {
    pub fn new(file_contents: &[u8]) -> Result<Self, MidiError> {
        let mut meta_data = Self {
            num_of_tracks: 0,
            file_format: FileFormat::SINGLE_TRACK,
            time_division: TimeDivision::PulsesPerQuarterNote(0),
        };

        validate_header_chunk_bytes(u32::from_be_bytes([file_contents[0], file_contents[1], file_contents[2], file_contents[3]]))?;

        meta_data.get_num_of_tracks(file_contents)?;
        meta_data.get_file_format(file_contents)?;
        meta_data.get_time_division(file_contents)?;

        Ok(meta_data)
    }

    fn get_num_of_tracks(&mut self, file_contents: &[u8]) -> Result<(), MidiError> {
        let num_of_track = &file_contents[NUM_OF_TRACKS_BYTES];
        let num_of_tracks = u16::from_be_bytes([num_of_track[0], num_of_track[1]]);
        match num_of_tracks {
            0 => Err(MidiError::ParseError(ParseError::InvalidNumOfTracks)),
            _ => {
                self.num_of_tracks = num_of_tracks;
                Ok(())
            }
        }
    }

    fn get_file_format(&mut self, file_contents: &[u8]) -> Result<(), MidiError> {
        let format_bytes = &file_contents[FILE_FORMAT_BYTES];
        let format_bytes = u16::from_be_bytes([format_bytes[0], format_bytes[1]]);
        
        match format_bytes {
            0 => {
                self.file_format = FileFormat::SINGLE_TRACK;
                Ok(())
            },
            1 => {
                self.file_format = FileFormat::MULTI_TRACK;
                Ok(())
            },
            2 => {
                self.file_format = FileFormat::MULTI_SONG;
                Ok(())
            },
            _ => Err(MidiError::ParseError(ParseError::InvalidFileFormat)),
        }
    }

    fn get_time_division(&mut self, file_contents: &[u8]) -> Result<(), MidiError> {
        let time_division = &file_contents[TIME_DIVISION_BYTES];
        let msb = 1 << 7;
        let smpte_frame_rate: i8 = time_division[0] as i8;

        if smpte_frame_rate & msb != 0 {
            let smp_timecode = (smpte_frame_rate & 0x7F) + -128;
            let fps = smp_timecode.abs() as u8;
            validate_fps_byte(fps)?;
            let ticks_per_frame = time_division[1];
            self.time_division = TimeDivision::SMPTE(fps, ticks_per_frame);
        } else {
            self.time_division = TimeDivision::PulsesPerQuarterNote(u16::from_be_bytes([time_division[0], time_division[1]]));
        }

        Ok(())
    }
}

fn validate_header_chunk_bytes(header_chunk_bytes: u32) -> Result<(), MidiError> {
    match header_chunk_bytes {
        HEADER_CHUNK_MTHD_BYTES => Ok(()),
        _ => Err(MidiError::ParseError(ParseError::InvalidHeader)),
    }
}

fn validate_fps_byte(fps_byte: u8) -> Result<(), MidiError> {
    match fps_byte {
        24 | 25 | 29 | 30 => Ok(()),
        _ => Err(MidiError::ParseError(ParseError::InvalidFPS))
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use env_logger;

    const HEADER_SIZE_BYTES: [u8; 4] = [0x00, 0x00, 0x00, 0x06];

    fn setup() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();
    }

    #[test]
    fn validate_header_chunk_bytes_success() {
        let header_chunk_bytes = u32::from_be_bytes([0x4D, 0x54, 0x68, 0x64]);
        let validate_result = validate_header_chunk_bytes(header_chunk_bytes);
        assert!(validate_result.is_ok());
    }

    #[test]
    fn validate_header_chunk_bytes_fail() {
        let header_chunk_bytes = u32::from_be_bytes([0x54, 0x68, 0x64, 0x00]);
        let validate_result = validate_header_chunk_bytes(header_chunk_bytes);
        assert!(validate_result.is_err());
        assert!(matches!(validate_result, Err(MidiError::ParseError(ParseError::InvalidHeader))));
    }

    #[test]
    fn validate_file_format_success_single_track() {
        let mut header_chunk_file_format = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_file_format.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_file_format.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x00, 0x60]);
        
        let metadata_result = MetaData::new(&header_chunk_file_format);
        assert!(metadata_result.is_ok());
        let metadata_result = metadata_result.unwrap();
        assert_eq!(metadata_result.file_format, FileFormat::SINGLE_TRACK);
    }
    
    #[test]
    fn validate_file_format_fail_invalid_track() {
        let mut header_chunk_file_format = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_file_format.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_file_format.extend_from_slice(&[0x00, 0x03, 0x00, 0x01, 0x00, 0x60]);

        let metadata_result = MetaData::new(&header_chunk_file_format);
        assert!(metadata_result.is_err());
        assert!(matches!(metadata_result, Err(MidiError::ParseError(ParseError::InvalidFileFormat))));
    }

    #[test]
    fn validate_num_of_tracks_success_01() {
        let mut header_chunk_num_of_tracks = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_num_of_tracks.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_num_of_tracks.extend_from_slice(&[0x00, 0x01, 0x00, 0x01, 0x00, 0x60]);

        let metadata_result = MetaData::new(&header_chunk_num_of_tracks);
        assert!(metadata_result.is_ok());
        let metadata_result = metadata_result.unwrap();
        assert_eq!(metadata_result.num_of_tracks, 1);
    }

    #[test]
    fn validate_num_of_tracks_fail_00() {
        let mut header_chunk_num_of_tracks = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_num_of_tracks.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_num_of_tracks.extend_from_slice(&[0x00, 0x01, 0x00, 0x00, 0x00, 0x06]);

        let metadata_result = MetaData::new(&header_chunk_num_of_tracks);
        assert!(metadata_result.is_err());
        assert!(matches!(metadata_result, Err(MidiError::ParseError(ParseError::InvalidNumOfTracks))));
    }

    #[test]
    fn validate_time_division_success_pulses() {
        let mut header_chunk_time_div: Vec<u8> = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_time_div.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_time_div.extend_from_slice(&[0x00, 0x01, 0x00, 0x01, 0x00, 0x60]);

        let metadata_result = MetaData::new(&header_chunk_time_div);
        assert!(metadata_result.is_ok());
        assert_eq!(metadata_result.unwrap().time_division, TimeDivision::PulsesPerQuarterNote(96));
    }

    #[test]
    fn validate_time_division_smpte_format_sucess() {
        setup();
        let mut header_chunk_time_div: Vec<u8> = HEADER_CHUNK_MTHD_BYTES.to_be_bytes().to_vec();
        header_chunk_time_div.extend_from_slice(&HEADER_SIZE_BYTES);
        header_chunk_time_div.extend_from_slice(&[0x00, 0x01, 0x00, 0x01, 0xE8, 0x08]);

        let metadata_result = MetaData::new(&header_chunk_time_div);
        assert!(metadata_result.is_ok());
        assert_eq!(metadata_result.unwrap().time_division, TimeDivision::SMPTE(24, 8));
    }
    
}