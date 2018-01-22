use parser::ParseTree;


pub fn adjust_and_validate(parse_tree: &mut ParseTree) -> bool
{
    let play_voices_match = play_voices_match(parse_tree);
    if !play_voices_match { return false }

    apply_octave_offsets(parse_tree);
    // Octave offsets
    // Stretch bars internally
    // Stretch bars to beat

    true
}


fn play_voices_match(parse_tree: &ParseTree) -> bool
{
    let mut valid = true;

    for piece in &parse_tree.pieces
    {
        for play in &piece.plays
        {
            let matches = piece.voices.iter().find(|voice| play.voice == Some(voice.name)).is_some();
            valid &= matches;
        }
    }

    valid
}


fn apply_octave_offsets(parse_tree: &mut ParseTree)
{
    use parser::NoteNode;

    for piece in &mut parse_tree.pieces
    {
        for play in &mut piece.plays
        {
            let voice = piece.voices.iter().find(|voice| play.voice == Some(voice.name)).unwrap();
            let octave = voice.octave.unwrap_or(0);
            let offset = octave * 12;

            for stave in &mut play.staves
            {
                for bar in &mut stave.bars
                {
                    for note in &mut bar.notes
                    {
                        if let &mut NoteNode::Note(midi) = note
                        {
                            *note = NoteNode::Note(midi + offset);
                        }
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;
    use parser::*;


    #[test]
    fn voices_match_pass()
    {
        let mut parse_tree = ParseTree
        {
            pieces: vec![
                PieceNode
                {
                    voices: vec![VoiceNode { name: "Drums", .. Default::default() }],
                    plays: vec![PlayNode { voice: Some("Drums"), .. Default::default() }],
                    .. Default::default()
                }
            ]
        };

        let valid = adjust_and_validate(&mut parse_tree);
        assert!(valid);
    }

    #[test]
    #[should_panic]
    fn voices_match_fail()
    {
        let mut parse_tree = ParseTree
        {
            pieces: vec![
                PieceNode
                {
                    voices: vec![VoiceNode { name: "Drums", .. Default::default() }],
                    plays: vec![PlayNode { voice: Some("Durms"), .. Default::default() }],
                    .. Default::default()
                }
            ]
        };

        let valid = adjust_and_validate(&mut parse_tree);
        assert!(valid);
    }

    #[test]
    fn test_octave_offset()
    {
        let mut parse_tree = ParseTree
        {
            pieces: vec![
                PieceNode
                {
                    voices: vec![VoiceNode { name: "Drums", octave: Some(2), .. Default::default() }],
                    plays: vec![PlayNode { voice: Some("Durms"), staves.. Default::default() }],
                    .. Default::default()
                }
            ]
        };
    }
}

