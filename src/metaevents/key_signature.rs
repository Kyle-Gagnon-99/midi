use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::{EventError, MidiError},
};

use super::{calculate_time_duration, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x59;

const KEY_SIGNATURE_SIZE: u8 = 5;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum Mode {
    Major,
    Minor,
}

impl Mode {
    fn to_number(&self) -> u8 {
        match self {
            Mode::Major => 0,
            Mode::Minor => 1,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum Accidentals {
    Flat,
    Natural,
    Sharp,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub enum Key {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct KeySignature {
    pub key: Key,
    pub accidental: Accidentals,
    pub mode: Mode,
    pub num_of_accidentals: i8,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct KeySignatureEvent {
    pub key_signature: KeySignature,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl KeySignature {
    pub fn new(key: Key, accidental: Accidentals, mode: Mode) -> Result<Self, MidiError> {
        Ok(Self {
            key,
            accidental,
            mode,
            num_of_accidentals: get_num_of_accidentals_from_key_signature(key, accidental),
        })
    }
}

impl Event for KeySignatureEvent {
    fn get_event_name(&self) -> String {
        String::from("Key Signature")
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
        let mut bytes: Vec<u8> = vec![
            METAEVENT_BYTE,
            METAEVENT_BYTE_TYPE,
            (KEY_SIGNATURE_SIZE - 3),
        ];
        bytes.push(self.key_signature.num_of_accidentals as u8);
        bytes.push(self.key_signature.mode.to_number());

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[
            METAEVENT_BYTE,
            METAEVENT_BYTE_TYPE,
            (KEY_SIGNATURE_SIZE - 3),
        ]);
        bytes.push(self.key_signature.num_of_accidentals as u8);
        bytes.push(self.key_signature.mode.to_number());

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for KeySignatureEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after 0xFF 0x59 0x02
        let data = &data[3..];

        // The first byte is how many accidentals it is
        let (key, accidental) = get_key_signature_from_num_of_accidentals(data[0] as i8)?;

        let mode: Mode = match data[1] {
            0 => Ok(Mode::Major),
            1 => Ok(Mode::Minor),
            _ => Err(MidiError::EventError(EventError::InvalidKeySignature(
                "Invalid mode type! Please use 0 for Major or 1 for Minor".to_string(),
            ))),
        }?;

        Ok(Self {
            key_signature: KeySignature::new(key, accidental, mode)?,
            event_size: KEY_SIGNATURE_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl KeySignatureEvent {
    pub fn new(key_signature: KeySignature) -> Result<Self, MidiError> {
        Ok(Self {
            key_signature,
            event_size: KEY_SIGNATURE_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}

pub(crate) fn get_key_signature_from_num_of_accidentals(
    byte: i8,
) -> Result<(Key, Accidentals), MidiError> {
    match byte {
        0 => Ok((Key::C, Accidentals::Natural)),
        x => {
            if x > 0 {
                match x {
                    1 => Ok((Key::G, Accidentals::Natural)),
                    2 => Ok((Key::D, Accidentals::Natural)),
                    3 => Ok((Key::A, Accidentals::Natural)),
                    4 => Ok((Key::E, Accidentals::Natural)),
                    5 => Ok((Key::B, Accidentals::Natural)),
                    6 => Ok((Key::F, Accidentals::Sharp)),
                    7 => Ok((Key::C, Accidentals::Sharp)),
                    _ => Err(MidiError::EventError(EventError::InvalidKeySignature(
                        format!(
                            "{} is an invalid Key Signature. It must be between -7 and 7",
                            x
                        ),
                    ))),
                }
            } else {
                match x {
                    -1 => Ok((Key::F, Accidentals::Natural)),
                    -2 => Ok((Key::B, Accidentals::Flat)),
                    -3 => Ok((Key::E, Accidentals::Flat)),
                    -4 => Ok((Key::A, Accidentals::Flat)),
                    -5 => Ok((Key::D, Accidentals::Flat)),
                    -6 => Ok((Key::G, Accidentals::Flat)),
                    -7 => Ok((Key::C, Accidentals::Flat)),
                    _ => Err(MidiError::EventError(EventError::InvalidKeySignature(
                        format!(
                            "{} is an invalid Key Signature. It must be between -7 and 7",
                            x
                        ),
                    ))),
                }
            }
        }
    }
}

fn get_num_of_accidentals_from_key_signature(key: Key, accidental: Accidentals) -> i8 {
    match (key, accidental) {
        (Key::C, Accidentals::Natural) => 0,
        (Key::G, Accidentals::Natural) => 1,
        (Key::D, Accidentals::Natural) => 2,
        (Key::A, Accidentals::Natural) => 3,
        (Key::E, Accidentals::Natural) => 4,
        (Key::B, Accidentals::Natural) => 5,
        (Key::F, Accidentals::Sharp) => 6,
        (Key::C, Accidentals::Sharp) => 7,
        (Key::F, Accidentals::Natural) => -1,
        (Key::B, Accidentals::Flat) => -2,
        (Key::E, Accidentals::Flat) => -3,
        (Key::A, Accidentals::Flat) => -4,
        (Key::D, Accidentals::Flat) => -5,
        (Key::G, Accidentals::Flat) => -6,
        (Key::C, Accidentals::Flat) => -7,

        // These are keys that aren't used at all but they are technically keys
        (Key::D, Accidentals::Sharp) => -3,
        (Key::E, Accidentals::Sharp) => -1,
        (Key::F, Accidentals::Flat) => 4,
        (Key::G, Accidentals::Sharp) => -4,
        (Key::A, Accidentals::Sharp) => -2,
        (Key::B, Accidentals::Sharp) => 0,
    }
}
