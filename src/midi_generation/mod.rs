use error::SourceMap;
use sequencing::data::*;


pub fn generate_midi(piece: &Piece, _source_map: &SourceMap) -> Option<Vec<u8>>
{
    use rimd::*;

    let smf = {
        const MICROSECONDS_PER_MIN: u32 = 60_000_000;
        const TICKS_PER_BEAT: i16 = 480;

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
            ];

            // TODO(***realname***): Handle overlapping notes:
            //      -   Break up each note into a start and end (remember end-1)
            //      -   Sort those by start time
            //      -   Use dt since last event as vtime
            let mut cursor = 0;
            for note in &voice.notes
            {
                let midi_note = note.midi.midi() as u8;
                let ticks_per_bar = TICKS_PER_BEAT as u64 * piece.beats;
                let ticks_per_division = ticks_per_bar / u64::from(voice.divisions_per_bar);
                let pos_ticks = ticks_per_division * u64::from(note.position);
                let len_ticks = ticks_per_division * u64::from(note.length);

                events.push(TrackEvent {
                    vtime: pos_ticks - cursor,
                    event: Event::Midi(MidiMessage::note_on(
                        midi_note,
                        100,
                        voice.channel - 1,
                    )),
                });

                events.push(TrackEvent {
                    vtime: len_ticks - 1,
                    event: Event::Midi(MidiMessage::note_off(midi_note, 0, voice.channel - 1)),
                });

                cursor = pos_ticks + len_ticks - 1;
            }

            tracks.push(Track {
                copyright: None,
                name: None,
                events,
            });
        }

        SMF {
            format: SMFFormat::MultiTrack,
            division: TICKS_PER_BEAT,
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
