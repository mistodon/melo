use crate::notes::Midi;
use crate::parsing::data::*;

pub fn stave(prefix: &str, notes: Vec<Vec<NoteNode>>) -> StaveNode {
    use std::borrow::Cow;

    StaveNode {
        prefix: Cow::Borrowed(prefix),
        bars: notes
            .into_iter()
            .map(|bar| {
                BarTypeNode::Bar(BarNode {
                    notes: bar,
                    note_locs: Vec::new(),
                })
            })
            .collect(),
        bar_locs: Vec::new(),
    }
}

pub fn midi(num: i8) -> Midi {
    Midi::from_raw(num).unwrap()
}
