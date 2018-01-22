use lexing::{ Token, Span, MetaToken };
use parsing::{ StaveNode, NoteNode };


pub fn mt(token: Token, span: (usize, usize)) -> MetaToken
{
    let span = Span(span.0, span.1);
    MetaToken { token, span }
}


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

