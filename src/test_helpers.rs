use lexer::{ Token, Span, MetaToken };


pub fn mt(token: Token, span: (usize, usize)) -> MetaToken
{
    let span = Span(span.0, span.1);
    MetaToken { token, span }
}

