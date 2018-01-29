use parsing::data::*;


pub fn stave(prefix: &str, notes: Vec<Vec<NoteNode>>) -> StaveNode
{
    use std::borrow::Cow;

    StaveNode {
        prefix: Cow::Borrowed(prefix),
        bars: notes
            .into_iter()
            .map(|bar| BarNode {
                notes: bar,
                note_locs: Vec::new(),
            })
            .collect(),
    }
}
