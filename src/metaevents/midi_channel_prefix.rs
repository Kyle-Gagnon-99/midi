use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x20;

const MIDI_CHANNEL_PREFIX_SIZE: u8 = 4;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct MidiChannelPrefixEvent {
    pub midi_channel: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for MidiChannelPrefixEvent {
    fn get_event_name(&self) -> String {
        String::from("Midi Channel Prefix")
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
        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE, 0x01];
        bytes.push(self.midi_channel);

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE, 0x20]);
        bytes.push(self.midi_channel);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for MidiChannelPrefixEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after 0xFF 0x20 0x01
        let data = &data[3..];

        // The MIDI channel is the next byte
        let midi_channel = data[0];

        Ok(Self {
            midi_channel,
            event_size: MIDI_CHANNEL_PREFIX_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl MidiChannelPrefixEvent {
    pub fn new(midi_channel: u8) -> Result<Self, MidiError> {
        Ok(Self {
            midi_channel,
            event_size: MIDI_CHANNEL_PREFIX_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}

mod tests {}
