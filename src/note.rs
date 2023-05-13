use std::marker::PhantomData;

use crate::metaevents::KeySignature;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PitchClass {
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note<'a> {
    pitch: PitchClass,
    octave: i8,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Note<'a> {
    pub fn new_from_midi_note_key_signature(midi_note: u8, key_signature: KeySignature) -> Self {
        let pitch_class_number = midi_note % 12;
        let octave = ((midi_note / 12) - 2) as i8;

        let sharp_pitch_classes = [
            PitchClass::C, PitchClass::CSharp, PitchClass::D, PitchClass::DSharp, PitchClass::E,
            PitchClass::F, PitchClass::FSharp, PitchClass::G, PitchClass::GSharp, PitchClass::A,
            PitchClass::ASharp, PitchClass::B
        ];

        let flat_pitch_classes = [
            PitchClass::C, PitchClass::DFlat, PitchClass::D, PitchClass::EFlat, PitchClass::E,
            PitchClass::F, PitchClass::GFlat, PitchClass::G, PitchClass::AFlat, PitchClass::A,
            PitchClass::BFlat, PitchClass::B
        ];

        let num_of_accidentals = key_signature.num_of_accidentals;

        let pitch_classes = if num_of_accidentals >= 0 {
            &sharp_pitch_classes
        } else {
            &flat_pitch_classes
        };

        Self {
            pitch: (pitch_classes[pitch_class_number as usize]),
            octave,
            _marker: PhantomData,
        }
    }

    pub fn new_from_midi_note(midi_note: u8) -> Self {
        let pitch_class_number = midi_note % 12;
        let octave = ((midi_note / 12) - 2) as i8;

        let sharp_pitch_classes = [
            PitchClass::C, PitchClass::CSharp, PitchClass::D, PitchClass::DSharp, PitchClass::E,
            PitchClass::F, PitchClass::FSharp, PitchClass::G, PitchClass::GSharp, PitchClass::A,
            PitchClass::ASharp, PitchClass::B
        ];

        Self {
            pitch: sharp_pitch_classes[pitch_class_number as usize],
            octave,
            _marker: PhantomData,
        }
    }

    pub fn to_midi_note(&self) -> u8 {
        let base_number = match self.pitch {
            PitchClass::C => 0,
            PitchClass::CSharp | PitchClass::DFlat => 1,
            PitchClass::D => 2,
            PitchClass::DSharp | PitchClass::EFlat => 3,
            PitchClass::E => 4,
            PitchClass::F => 5,
            PitchClass::FSharp | PitchClass::GFlat => 6,
            PitchClass::G => 7,
            PitchClass::GSharp | PitchClass::AFlat => 8,
            PitchClass::A => 9,
            PitchClass::ASharp | PitchClass::BFlat => 10,
            PitchClass::B => 11,
        };

        12_u8 * self.octave as u8 + base_number as u8
    }
}