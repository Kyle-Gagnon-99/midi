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

const MIDI_EVENT_TYPE: u8 = 0xA0;

const PITCH_BEND_CHANGE_SIZE: u8 = 0x03;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PitchBendChangeEvent {
    pub pitch_bend_lsb: u8,
    pub pitch_bend_msb: u8,
    pub channel: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for PitchBendChangeEvent {
    fn get_event_name(&self) -> String {
        String::from("Pitch Bend Change")
    }

    fn is_running_status_allowed(&self) -> bool {
        true
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
        let bytes: Vec<u8> = vec![
            (MIDI_EVENT_TYPE | self.channel),
            self.pitch_bend_lsb,
            self.pitch_bend_msb,
        ];

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[
            (MIDI_EVENT_TYPE | self.channel),
            self.pitch_bend_lsb,
            self.pitch_bend_msb,
        ]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for PitchBendChangeEvent {
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
        // The first four bits (nibble) say what the event is
        // THe last four bits say what channel
        let channel = data[0] & CHANNEL_MASK;

        // The next byte is what midi note this is applied to
        let pitch_bend_lsb = data[1];

        // The final byte is the value of the pressure
        let pitch_bend_msb = data[2];

        Ok(Self {
            pitch_bend_lsb,
            pitch_bend_msb,
            channel,
            event_size: PITCH_BEND_CHANGE_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}
