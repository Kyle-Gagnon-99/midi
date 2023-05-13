use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x54;

const SMPTE_OFFSET_SIZE: u8 = 8;

/// Used to store the SMPTE Offset Event.
/// This event is not really used in this library but it is being
/// stored to keep the integrity of the MIDI file
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SMPTEOffsetEvent {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub frame_rate: u8,
    pub fractional_frames: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for SMPTEOffsetEvent {
    fn get_event_name(&self) -> String {
        String::from("SMPTE Offset")
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
        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE, (SMPTE_OFFSET_SIZE - 3)];
        bytes.extend_from_slice(&[
            self.hour,
            self.minute,
            self.second,
            self.frame_rate,
            self.fractional_frames,
        ]);
        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE, (SMPTE_OFFSET_SIZE - 3)]);
        bytes.extend_from_slice(&[
            self.hour,
            self.minute,
            self.second,
            self.frame_rate,
            self.fractional_frames,
        ]);
        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for SMPTEOffsetEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after 0xFF 0x54 0x05
        let data = &data[3..];

        // The first byte is the hour
        let hour = data[0];

        // The next byte is the minute
        let minute = data[1];

        // The next byte is the second
        let second = data[2];

        // The byte is the frame rate
        let frame_rate = data[3];

        // The final byte is the fractional frame
        let fractional_frames = data[4];

        Ok(Self {
            hour,
            minute,
            second,
            frame_rate,
            fractional_frames,
            event_size: SMPTE_OFFSET_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}
