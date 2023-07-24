#[test]
#[cfg(feature = "json")]
fn view_midi_file_data() {
    use std::fs::File;
    use std::io::Write;

    let midi_file = midi::Midi::new("midi_testing.mid");
    
    let json_text = midi_file.unwrap().to_json().unwrap();

    let mut file = File::create("output.json").expect("Unable to create file");
    write!(file, "{}", json_text).expect("Unable to write data");
}