#[warn(missing_docs)]

#[cfg(feature = "json")]
extern crate serde;

#[cfg(feature = "json")]
use serde::Serialize;

use std::{path::Path, fs::File, io::Read};
use metadata::MetaData;
use metaevents::TimeSignature;
use midi_error::MidiError;
use track::Track;

use hex::encode_upper;
use log::debug;

pub mod metadata;
pub mod track;
pub mod events;
pub mod metaevents;
pub mod midi_error;
pub mod messages;
pub mod note;

#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Midi {
    pub midi_file: String,
    pub meta_data: MetaData,
    pub tempo: u32,
    pub time_signature: TimeSignature,
    pub track_list: Vec<Track>,
}

impl Midi {
    pub fn new(midi_file: &'static str) -> Result<Self, MidiError> {
        let tempo: u32 = 120;
        let path = Path::new(midi_file);

        let mut file_contents = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut file_contents)?;

        let meta_data = MetaData::new(&file_contents[0..14])?;
        let track_list = Track::get_track_list(&file_contents[14..])?;

        let midi_struct = Self {
            midi_file: String::from(midi_file),
            meta_data,
            tempo,
            time_signature: TimeSignature::new(4, 4)?,
            track_list,
        };

        Ok(midi_struct)
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

}

pub fn print_file_contents(file_conents: &[u8]) {
    let hex_lines = file_conents.chunks(8)
        .enumerate()
        .map(|(i, chunk)| {
            let hex_string = encode_upper(chunk);
            format!("{:04X}: {}\n", i * 8, hex_string)
        })
        .collect::<String>();
    debug!("The contents of the midi file is as follows:\n{}", hex_lines);
}

pub(crate) fn is_msb_zero(byte: u8) -> bool {
    (byte & 0b1000_0000) == 0
}

#[cfg(test)]
mod midi_lib_tests {

}
