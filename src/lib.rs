//! A simple crate for converting the MIDI output of the Novation ReMOTE 25SL into user-friendly
//! rust-esque types.

pub extern crate pitch_calc;
pub use pitch_calc::{Letter, LetterOctave};

// The names of the ports on which the `25SL` emits MIDI input values.
pub const MIDI_INPUT_PORT_0: &'static str = "ReMOTE SL 24:0";
pub const MIDI_INPUT_PORT_1: &'static str = "ReMOTE SL 24:1";
pub const MIDI_INPUT_PORT_2: &'static str = "ReMOTE SL 24:2";

/// The MIDI input ports on which the `25SL` emits MIDI input values.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InputPort {
    /// Receives keyboard note events as well as `Pitch` and `Mod` `Control` events.
    A,
    /// Receives all other `Control` events.
    B,
    /// Receives messages about newly loaded presets.
    C,
}

/// All possible events that might be emitted from the ReMOTE 25SL.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Event {
    Control(Control),
    Key(State, LetterOctave, u8),
}

/// Note events emitted from key presses.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum State {
    /// The note was pressed.
    On,
    /// The note was released.
    Off
}

/// Most controls on the 25SL come in 8 strips.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Oct { A, B, C, D, E, F, G, H }

/// The 4 distinct rows on which `Button`s are placed.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ButtonRow {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

/// Axes on which the `TouchPad` can output values.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Axis { X, Y }

/// The left and right sides of the controller.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// The page up and down buttons.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Page {
    Up,
    Down,
}

/// The four buttons on the upper left hand side of the controller from top to bottom.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LeftButton { A, B, C, D }

/// The three buttons on the upper right hand side of the controller from top to bottom.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RightButton { A, B, C }

/// Media playback-style control buttons.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Playback {
    Previous,
    Next,
    Stop,
    Play,
    Loop,
    Record,
}

/// Controller events.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Control {

    /// A magnitude in the direction in which the dial was turned.
    ///
    /// Values may range from -64 to 64 (exclusive).
    RotaryDial(Oct, i8),

    /// The value to which the slider was set.
    ///
    /// Values range from `0` to `127` (inclusive).
    RotarySlider(Oct, u8),

    /// The value to which the slider was set.
    ///
    /// Values range from `0` to `127` (inclusive).
    VerticalSlider(Oct, u8),

    /// The force with which the pad was pressed.
    ///
    /// Values range from `0` to `127` (inclusive).
    PressurePad(Oct, u8),

    /// A button was pressed on the given row.
    Button(ButtonRow, Oct, State),

    /// The position on the touch pad that was pressed.
    ///
    /// Values range from `0` to `127` (inclusive).
    TouchPad(Axis, u8),

    /// The position of the pitch bender.
    ///
    /// Ranges from -64 to 64 (exclusive).
    Pitch(i8),

    /// The position of the modulation bender.
    ///
    /// Values range from 0 to 127 (inclusive).
    Mod(u8),

    /// The page up and down buttons on the top left and right of the controller..
    Page(Side, Page, State),

    /// The four buttons on the upper left hand side of the controller.
    LeftButton(LeftButton, State),

    /// The three buttons on the upper right hand side of the controller.
    RightButton(RightButton, State),

    /// Media playback-style control buttons.
    Playback(Playback, State),
}


impl Oct {

    /// Create an `Oct` from the given value where 0 == A, 1 == B, etc.
    fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Oct::A),
            1 => Some(Oct::B),
            2 => Some(Oct::C),
            3 => Some(Oct::D),
            4 => Some(Oct::E),
            5 => Some(Oct::F),
            6 => Some(Oct::G),
            7 => Some(Oct::H),
            _ => None,
        }
    }

}


impl InputPort {

    /// Determine the `InputPort` from its name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            MIDI_INPUT_PORT_0 => Some(InputPort::A),
            MIDI_INPUT_PORT_1 => Some(InputPort::B),
            MIDI_INPUT_PORT_2 => Some(InputPort::C),
            _ => None,
        }
    }

}


impl Event {

    /// Produce an `Event` from the given MIDI input port number and the MIDI message itself.
    pub fn from_midi(port: InputPort, msg: &[u8]) -> Option<Self> {
        match port {

            // Receive keyboard note events and pitch/mod bend values.
            InputPort::A => match msg.len() {
                3 => match (msg[0], msg[1], msg[2]) {

                    // Pitch bend.
                    (224, 0, pitch) => Some(Control::Pitch(pitch as i8 - 64).into()),

                    // Modulation bend.
                    (176, 1, modulation) => Some(Control::Mod(modulation).into()),

                    // Notes pressed on the keyboard.
                    (state, step, velocity) => {
                        let letter_octave = pitch_calc::Step(step as f32).to_letter_octave();
                        let note = match state {
                            144 => Some(State::On),
                            128 => Some(State::Off),
                            _ => None,
                        };
                        note.map(|note| Event::Key(note, letter_octave, velocity))
                    },

                },
                _ => None,
            },

            // Receive control events.
            InputPort::B => match msg.len() {
                3 => match (msg[0], msg[1], msg[2]) {

                    // Rotary dialers.
                    (176, n @ 56...63, value) => {
                        let oct = Oct::from_u8(n - 56).unwrap();
                        let value = if value > 64 { -(value as i8 - 64) } else { value as i8 };
                        Some(Control::RotaryDial(oct, value).into())
                    },

                    // Rotary sliders.
                    (176, n @ 8...15, value) => {
                        let oct = Oct::from_u8(n - 8).unwrap();
                        Some(Control::RotarySlider(oct, value).into())
                    },

                    // Vertical sliders.
                    (176, n @ 16...23, value) => {
                        let oct = Oct::from_u8(n - 16).unwrap();
                        Some(Control::VerticalSlider(oct, value).into())
                    },

                    // Pressure pads.
                    (144, n @ 36...43, velocity) => {
                        let oct = Oct::from_u8(n - 36).unwrap();
                        Some(Control::PressurePad(oct, velocity).into())
                    },

                    // Touch pad.
                    (176, axis @ 68...69, value) => {
                        let axis = if axis == 68 { Axis::X } else { Axis::Y };
                        Some(Control::TouchPad(axis, value).into())
                    },


                    ///////////////////
                    ///// Buttons /////
                    ///////////////////

                    // Top left row buttons.
                    (176, n @ 24...31, state) => {
                        let oct = Oct::from_u8(n - 24).unwrap();
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Button(ButtonRow::TopLeft, oct, state).into())
                    },

                    // Bottom left row buttons.
                    (176, n @ 32...39, state) => {
                        let oct = Oct::from_u8(n - 32).unwrap();
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Button(ButtonRow::BottomLeft, oct, state).into())
                    },

                    // Top right row buttons.
                    (176, n @ 40...47, state) => {
                        let oct = Oct::from_u8(n - 40).unwrap();
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Button(ButtonRow::TopRight, oct, state).into())
                    },

                    // Bottom right row buttons.
                    (176, n @ 48...55, state) => {
                        let oct = Oct::from_u8(n - 48).unwrap();
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Button(ButtonRow::BottomRight, oct, state).into())
                    },

                    // Page up and down.
                    (176, n @ 88...91, state) => {
                        let (side, page) = match n {
                            88 => (Side::Left, Page::Up),
                            89 => (Side::Left, Page::Down),
                            90 => (Side::Right, Page::Up),
                            91 => (Side::Right, Page::Down),
                            _ => unreachable!(),
                        };
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Page(side, page, state).into())
                    },

                    // Left-hand side buttons.
                    (176, n @ 80...83, state) => {
                        let button = match n {
                            80 => LeftButton::A,
                            81 => LeftButton::B,
                            82 => LeftButton::C,
                            83 => LeftButton::D,
                            _ => unreachable!(),
                        };
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::LeftButton(button, state).into())
                    },

                    // Right-hand side buttons.
                    (176, n @ 85...87, state) => {
                        let button = match n {
                            85 => RightButton::A,
                            86 => RightButton::B,
                            87 => RightButton::C,
                            _ => unreachable!(),
                        };
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::RightButton(button, state).into())
                    },

                    // Playback buttons.
                    (176, n @ 72...77, state) => {
                        let playback = match n {
                            72 => Playback::Previous,
                            73 => Playback::Next,
                            74 => Playback::Stop,
                            75 => Playback::Play,
                            76 => Playback::Record,
                            77 => Playback::Loop,
                            _ => unreachable!(),
                        };
                        let state = if state == 0 { State::Off } else { State::On };
                        Some(Control::Playback(playback, state).into())
                    },

                    _ => None,

                },
                _ => None,
            },

            // Receive preset state loaded from the controller.
            InputPort::C => {
                None
            },
        }
    }

}


impl From<Control> for Event {
    fn from(control: Control) -> Self {
        Event::Control(control)
    }
}
