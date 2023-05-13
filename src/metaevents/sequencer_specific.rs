use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_bytes_to_vlq, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x7F;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SequencerSpecificEvent {
    pub data: Vec<u8>,
    pub data_length: u32,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for SequencerSpecificEvent {
    fn get_event_name(&self) -> String {
        String::from("Sequence Specific")
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

    fn get_delta_time(&self) -> u32 {
        self.delta_time
    }

    fn get_current_time(&self) -> Instant {
        self.current_time
    }

    fn get_time_duration(&self) -> Duration {
        self.time_duration
    }

    fn to_bytes(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE];
        bytes.extend_from_slice(&from_vlq_to_bytes(self.data_length));
        bytes.extend_from_slice(&self.data);

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE]);
        bytes.extend_from_slice(&from_vlq_to_bytes(self.data_length));
        bytes.extend_from_slice(&self.data);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for SequencerSpecificEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after the 0xFF 0x7F bytes
        let data = &data[2..];

        // The length of the data that follows is stored in VLQ
        let (data_length, num_of_bytes) = from_bytes_to_vlq(data);

        // Grab the data
        // We don't care what the data actually is, we just need to extract it
        let data = &data[(num_of_bytes as usize)..((num_of_bytes as u32 + data_length) as usize)];

        // The event size is 0xFF 0x7F bytes plus the number of bytes from the VLQ size + the length of the data
        let event_size = 2 + num_of_bytes + data.len() as u8;

        Ok(Self {
            data: data.to_vec(),
            data_length,
            event_size,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}
