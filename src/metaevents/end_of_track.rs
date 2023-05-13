use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x2F;

const END_OF_TRACK_SIZE: u8 = 3;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct EndOfTrack {
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for EndOfTrack {
    fn get_event_name(&self) -> String {
        String::from("End of Track")
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
        Ok(vec![
            METAEVENT_BYTE,
            METAEVENT_BYTE_TYPE,
            (END_OF_TRACK_SIZE - 3),
        ])
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE, (END_OF_TRACK_SIZE - 3)]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for EndOfTrack {
    type Output = Self;

    #[allow(unused_variables)]
    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        Ok(Self {
            event_size: END_OF_TRACK_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl EndOfTrack {
    pub fn new() -> Result<Self, MidiError> {
        Ok(Self {
            event_size: END_OF_TRACK_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}
