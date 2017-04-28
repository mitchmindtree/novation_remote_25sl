extern crate novation_remote_25sl;
extern crate midir;


fn main() {
    let midi_in = midir::MidiInput::new("Ableton Novation ReMOTE 25SL").unwrap();

    // A channel for sending events to the main thread.
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let mut inputs = Vec::new();

    // For each point used by the 25SL, check for events.
    for i in 0..midi_in.port_count() {
        let name = midi_in.port_name(i).unwrap();
        if let Some(port) = novation_remote_25sl::InputPort::from_name(&name) {
            let event_tx = event_tx.clone();
            let midi_in = midir::MidiInput::new(&name).unwrap();
            let input = midi_in.connect(i, "ReMOTE 25SL: 0", move |_stamp, msg, _| {
                if let Some(event) = novation_remote_25sl::Event::from_midi(port, msg) {
                    event_tx.send(event).unwrap();
                }
            }, ()).unwrap();
            inputs.push(input);
        }
    }

    for event in event_rx {
        println!("{:?}", &event);
    }

    for input in inputs {
        input.close();
    }
}
