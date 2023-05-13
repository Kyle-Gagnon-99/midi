use std::{sync::Once, env, fs::File, io::Write};

use log::debug;

static INIT: Once = Once::new();

fn setup() {
    env::set_var("RUST_LOG", "debug");
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(false).try_init();
    });
}

#[test]
#[cfg(feature = "json")]
fn view_midi_file_data() {
    setup();
    let midi_file = midi::Midi::new("midi_testing.mid");
    
    let json_text = midi_file.unwrap().to_json().unwrap();

    let mut file = File::create("output.json").expect("Unable to create file");
    write!(file, "{}", json_text).expect("Unable to write data");
}