extern crate midscript;

#[macro_use]
extern crate pretty_assertions;


#[test]
fn test_parsing()
{
    use midscript::lexing;
    use midscript::parsing;

    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/parsing_test.midscript"));

    let tokens = lexing::lex(source).unwrap();
    let result = parsing::parse(&tokens);

    let piece = &result.pieces[0];
    assert_eq!(piece.title.unwrap(), "Parser Test");
    assert_eq!(piece.composer.unwrap(), "***Realname*** ***Lastname***");
    assert_eq!(piece.tempo.unwrap(), 160);
    assert_eq!(piece.beats.unwrap(), 4);

    let lead_voice = &piece.voices[0];
    assert_eq!(lead_voice.name, "Lead");
    assert_eq!(lead_voice.program.unwrap(), 29);
    assert_eq!(lead_voice.channel.unwrap(), 1);

    let drum_voice = &piece.voices[1];
    assert_eq!(drum_voice.name, "Drums");
    assert_eq!(drum_voice.channel.unwrap(), 10);
    assert_eq!(drum_voice.octave.unwrap(), -2);

    let play_drums = &piece.plays[0];
    assert_eq!(play_drums.voice.unwrap(), "Drums");
    assert_eq!(play_drums.staves.len(), 3);
    assert_eq!(play_drums.staves[0].bars.len(), 2);
    assert_eq!(play_drums.staves[0].bars[0].notes.len(), 16);

    let play_lead = &piece.plays[1];
    assert_eq!(play_lead.voice.unwrap(), "Lead");
    assert_eq!(play_lead.staves.len(), 1);
    assert_eq!(play_lead.staves[0].bars.len(), 2);
    assert_eq!(play_lead.staves[0].bars[0].notes.len(), 2);
}

