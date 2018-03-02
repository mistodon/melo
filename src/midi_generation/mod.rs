pub mod data;


use self::data::*;


use error::SourceMap;
use sequencing::data::*;


pub fn generate_midi(
    piece: &Piece,
    _source_map: &SourceMap,
    options: &MidiGenerationOptions,
) -> Option<Vec<u8>>
{
    use rimd::*;

    let smf = {
        const MICROSECONDS_PER_MIN: u32 = 60_000_000;
        const VEL_FIRST: u8 = 105;
        const VEL_STRONG: u8 = 95;
        const VEL_WEAK: u8 = 80;

        let tempo = MICROSECONDS_PER_MIN / piece.tempo as u32;

        let track0 = Track {
            copyright: None,
            name: None,
            events: vec![
                TrackEvent {
                    vtime: 0,
                    event: Event::Meta(MetaEvent::text_event("note track".into())),
                },
                TrackEvent {
                    vtime: 0,
                    event: Event::Meta(MetaEvent::tempo_setting(tempo)),
                },
                TrackEvent {
                    vtime: 0,
                    event: Event::Meta(MetaEvent::key_signature(0, 0)),
                },
                TrackEvent {
                    vtime: 0,
                    event: Event::Meta(MetaEvent::time_signature(piece.beats as u8, 2, 48, 8)),
                },
            ],
        };

        let mut tracks = vec![track0];

        for voice in &piece.voices
        {
            // Using +0.5 instead of round for wasm compatibility.
            let volume = ((voice.volume.unwrap_or(1.0) * 127.0) + 0.5) as u8;

            let mut events = vec![
                TrackEvent {
                    vtime: 0,
                    event: Event::Meta(MetaEvent::text_event("note track".into())),
                },
                TrackEvent {
                    vtime: 1,
                    event: Event::Midi(MidiMessage::program_change(
                        voice.program,
                        voice.channel - 1,
                    )),
                },
                TrackEvent {
                    vtime: 0,
                    event: Event::Midi(MidiMessage::control_change(
                        7,
                        volume,
                        voice.channel - 1,
                    )),
                },
            ];

            let split_notes = {
                let mut split_notes = Vec::new();

                for note in &voice.notes
                {
                    let midi_note = note.midi.midi() as u8;
                    let ticks_per_bar = options.ticks_per_beat as u64 * piece.beats;
                    let ticks_per_division =
                        ticks_per_bar / u64::from(voice.divisions_per_bar);
                    let pos_ticks = ticks_per_division * u64::from(note.position);
                    let len_ticks = ticks_per_division * u64::from(note.length);
                    let vel = {
                        let divisions_per_beat = voice.divisions_per_bar / piece.beats as u32;
                        let divisions_per_beat = ::std::cmp::max(divisions_per_beat, 1);

                        if note.position % voice.divisions_per_bar == 0
                        {
                            VEL_FIRST
                        }
                        else if note.position % divisions_per_beat == 0
                        {
                            VEL_STRONG
                        }
                        else
                        {
                            VEL_WEAK
                        }
                    };

                    let note_on = (true, midi_note, pos_ticks, vel);
                    let note_off = (false, midi_note, pos_ticks + len_ticks - 1, 0);
                    split_notes.push(note_on);
                    split_notes.push(note_off);
                }

                // Sort by start time
                split_notes.sort_by_key(|note| note.2);

                split_notes
            };

            let mut cursor = 0;
            for note in split_notes
            {
                let (on, midi_note, pos_ticks, vel) = note;
                let vtime = pos_ticks - cursor;
                let message = {
                    if on
                    {
                        MidiMessage::note_on(midi_note, vel, voice.channel - 1)
                    }
                    else
                    {
                        MidiMessage::note_off(midi_note, vel, voice.channel - 1)
                    }
                };

                events.push(TrackEvent {
                    vtime,
                    event: Event::Midi(message),
                });

                cursor = pos_ticks;
            }

            tracks.push(Track {
                copyright: None,
                name: None,
                events,
            });
        }

        SMF {
            format: SMFFormat::MultiTrack,
            division: options.ticks_per_beat,
            tracks,
        }
    };

    let writer = SMFWriter::from_smf(smf);

    let buffer = {
        let mut buffer = Vec::new();
        writer.write_all(&mut buffer).ok()?;
        buffer
    };

    Some(buffer)
}
