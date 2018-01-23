use parsing::data::*;


pub fn adjust_and_validate(parse_tree: &mut ParseTree) -> bool
{
    let play_voices_match = play_voices_match(parse_tree);
    if !play_voices_match { return false }

    let staves_all_same_length = staves_all_same_length(parse_tree);
    if !staves_all_same_length { return false }

    apply_octave_offsets(parse_tree);

    let aligned = normalize_bar_lengths(parse_tree);
    if !aligned { return false }

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


// TODO(***realname***): Could do this in parsing
fn staves_all_same_length(parse_tree: &ParseTree) -> bool
{
    for piece in &parse_tree.pieces
    {
        for play in &piece.plays
        {
            if !play.staves.is_empty()
            {
                let stave_length = play.staves[0].bars.len();
                return play.staves.iter().all(|stave| stave.bars.len() == stave_length)
            }
        }
    }
    true
}


fn apply_octave_offsets(parse_tree: &mut ParseTree)
{
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


fn stretch(bar: &mut BarNode, length: usize) -> Result<(), ()>
{
        let prev_length = bar.notes.len();
        let can_stretch = length % prev_length == 0;

        if !can_stretch
        {
            return Err(())
        }

        let stride = length / prev_length;

        let result = (0..length).into_iter()
            .map(|beat| if beat % stride == 0 { bar.notes[beat / stride] } else { NoteNode::Rest })
            .collect();

        bar.notes = result;
        Ok(())
}


fn normalize_bar_lengths(parse_tree: &mut ParseTree) -> bool
{
    for piece in &mut parse_tree.pieces
    {
        let mut max_bar_len = 0;

        // TODO(***realname***): Technically could do this in parse_play
        for play in &mut piece.plays
        {
            let staves = &mut play.staves;
            if staves.is_empty()
            {
                continue
            }

            let stave_length = staves[0].bars.len();
            for bar_index in 0..stave_length
            {
                let bar_length = staves.iter()
                    .map(|stave| stave.bars[bar_index].notes.len())
                    .max()
                    .unwrap();
                max_bar_len = ::std::cmp::max(max_bar_len, bar_length);

                for stave in staves.iter_mut()
                {
                    if stretch(&mut stave.bars[bar_index], bar_length).is_err()
                    {
                        return false
                    }
                }
            }
        }

        let min_bar_len = {
            // TODO(***realname***): This should really be the LCM of max_bar_len and piece.beats
            let beats = piece.beats.unwrap_or(4) as usize;
            let min_bar_len = ::std::cmp::max(max_bar_len, beats);
            let min_bar_len = if min_bar_len % beats == 0 { min_bar_len } else { min_bar_len * beats };
            min_bar_len
        };

        for play in &mut piece.plays
        {
            for stave in &mut play.staves
            {
                for bar in &mut stave.bars
                {
                    if stretch(bar, min_bar_len).is_err()
                    {
                        return false
                    }
                }
            }
        }
    }

    true
}


#[cfg(test)]
mod tests
{
    use super::*;
    use test_helpers::stave;


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
                    plays: vec![
                        PlayNode
                        {
                            voice: Some("Drums"),
                            staves: vec![
                                stave("C", vec![vec![NoteNode::Note(60)]])
                            ],
                            .. Default::default()
                        }],
                    .. Default::default()
                }
            ]
        };

        adjust_and_validate(&mut parse_tree);
        assert_eq!(parse_tree.pieces[0].plays[0].staves[0].bars[0].notes[0], NoteNode::Note(84));
    }

    // TODO(***realname***): Finish writing tests for this
}

