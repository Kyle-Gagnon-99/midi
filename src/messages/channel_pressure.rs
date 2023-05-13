use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    metaevents::{calculate_time_duration, from_vlq_to_bytes},
    midi_error::MidiError,
};

use super::CHANNEL_MASK;

const MIDI_EVENT_TYPE: u8 = 0xD0;

const CHANNEL_PRESSURE_SIZE: u8 = 0x02;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct ChannelPressureEvent {
    pub pressure_value: u8,
    pub channel: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for ChannelPressureEvent {
    fn get_event_name(&self) -> String {
        String::from("Channel Pressure")
    }

    fn is_running_status_allowed(&self) -> bool {
        false
    }

    fn event_type(&self) -> u8 {
        MIDI_EVENT_TYPE
    }

    fn get_channel(&self) -> u8 {
        self.channel
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
        let bytes: Vec<u8> = vec![(MIDI_EVENT_TYPE | self.channel), self.pressure_value];

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[(MIDI_EVENT_TYPE | self.channel), self.pressure_value]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for ChannelPressureEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts at the beginning
        let data = &data[0..];

        // The first byte is the status byte
        // The first four bits say what event this is (which we don't care about now)
        // then the last four bits give us the channel number
        let channel = data[0] & CHANNEL_MASK;

        // The next byte is the program change value
        let pressure_value = data[1];

        Ok(Self {
            pressure_value,
            channel,
            event_size: CHANNEL_PRESSURE_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}
