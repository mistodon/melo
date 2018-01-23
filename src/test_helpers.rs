use parsing::{ StaveNode, NoteNode };


pub fn stave(prefix: &str, notes: Vec<Vec<NoteNode>>) -> StaveNode
{
    use std::borrow::Cow;
    use parsing::{ BarNode };

    StaveNode
    {
        prefix: Cow::Borrowed(prefix),
        bars: notes.into_iter().map(|bar| BarNode { notes: bar }).collect()
    }
}

