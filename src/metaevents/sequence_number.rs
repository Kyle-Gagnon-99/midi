use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::events::{Event, FromBytes};
use crate::metadata::TimeDivision;
use crate::midi_error::{MidiError, ParseError};

use super::{calculate_time_duration, from_vlq_to_bytes};

const METAEVENT_BYTE_TYPE: u8 = 0x20;

const SEQUENCE_NUMBER_SIZE: u8 = 0x00;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SequenceNumber {
    pub sequence_number: u16,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for SequenceNumber {
    fn get_event_name(&self) -> String {
        String::from("Sequence Number")
    }

    fn is_running_status_allowed(&self) -> bool {
        false
    }

    fn event_type(&self) -> u8 {
        METAEVENT_BYTE_TYPE
    }

    fn get_channel(&self) -> u8 {
        0
    }

    fn get_event_size(&self) -> u8 {
        self.event_size
    }

    fn get_current_time(&self) -> Instant {
        self.current_time
    }

    fn get_delta_time(&self) -> u32 {
        self.delta_time
    }

    fn get_time_duration(&self) -> Duration {
        self.time_duration
    }

    fn to_bytes(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = vec![0xFF, 0x00, 0x02];
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&from_vlq_to_bytes(self.delta_time));
        bytes.extend_from_slice(&[0xFF, 0x00, 0x02]);
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for SequenceNumber {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self, MidiError> {
        // Do some basic error checking
        if data.len() != SEQUENCE_NUMBER_SIZE as usize {
            return Err(MidiError::ParseError(ParseError::InvalidEventBytes(
                String::from("Sequence Number Event Error: Invalid Size"),
            )));
        }

        if data[1] != 0x00 || data[2] != 0x02 {
            return Err(MidiError::ParseError(ParseError::InvalidEventBytes(
                String::from("Sequence Number Event Error: Invalid Bytes"),
            )));
        }

        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // Start after the 0xFF 0x00 and 0x02 bytes
        let data = &data[3..];

        // The sequence number is the next two bytes
        let sequence_number = u16::from_be_bytes([data[0], data[1]]);

        Ok(Self {
            sequence_number,
            event_size: SEQUENCE_NUMBER_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl SequenceNumber {
    pub fn new(sequence_number: u16) -> Result<Self, MidiError> {
        Ok(Self {
            sequence_number,
            event_size: SEQUENCE_NUMBER_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}

#[cfg(test)]
mod sequence_number_tests {

    use super::*;

    #[test]
    fn create_sequence_number_event_new_success() {
        let test_number: u16 = 5;
        let result = SequenceNumber::new(test_number);
        assert!(result.is_ok());
        let sequnece_number = result.unwrap();
        assert_eq!(sequnece_number.sequence_number, test_number)
    }

    #[test]
    fn create_sequence_number_event_data_success() {
        let test_number: Vec<u8> = vec![0x00, 0x05];
        let mut data = vec![0xFF, 0x00, 0x02];
        data.extend_from_slice(&test_number);
        let result =
            SequenceNumber::from_bytes(&data, 0, TimeDivision::PulsesPerQuarterNote(96), 120);
        assert!(result.is_ok());

        let seq_num = result.unwrap();
        assert_eq!(
            seq_num.sequence_number,
            u16::from_be_bytes([test_number[0], test_number[1]])
        );
    }
}
