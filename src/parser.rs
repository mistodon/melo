use std::iter::Peekable;
use std::slice::Iter;

use lexer::{ Token, Span, MetaToken };
use lexer::Token::*;


type TokenStream<'a> = Peekable<Iter<'a, MetaToken<'a>>>;


#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree<'a>
{
    pub pieces: Vec<PieceNode<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PieceNode<'a>
{
    pub title: Option<&'a str>,
    pub composer: Option<&'a str>,
}


pub fn parse<'a>(tokens: &[MetaToken<'a>]) -> ParseTree<'a>
{
    let mut pieces = Vec::new();
    let mut stream = tokens.iter().peekable();

    match stream.peek()
    {
        Some(&&MetaToken { token: Piece, .. }) => {
            while stream.peek().is_some()
            {
                pieces.push(parse_piece(&mut stream));
            }
        }
        _ => {
            pieces.push(parse_piece_from_body(&mut stream));
        }
    }

    assert!(stream.next().is_none());

    ParseTree { pieces }
}


fn expect_token(stream: &mut TokenStream, token: Token)
{
    assert_eq!(stream.next().map(|meta| meta.token), Some(token))
}

fn skip_token(stream: &mut TokenStream, token: Token)
{
    if stream.peek().map(|meta| meta.token) == Some(token)
    {
        stream.next();
    }
}


fn parse_piece<'a>(stream: &mut TokenStream) -> PieceNode<'a>
{
    expect_token(stream, Piece);
    expect_token(stream, LeftBrace);

    let piece_node = parse_piece_from_body(stream);

    expect_token(stream, RightBrace);

    piece_node
}

fn parse_piece_from_body<'a>(stream: &mut TokenStream) -> PieceNode<'a>
{
    let mut piece_node = PieceNode::default();

    loop
    {
        match stream.peek()
        {
            Some(&&MetaToken { token: RightBrace, .. }) => return piece_node,
            None => return piece_node,
            Some(&&MetaToken { token: Voice, ..}) => unimplemented!(),
            Some(&&MetaToken { token: Play, ..}) => unimplemented!(),
        }

        let (key_name, value) = parse_key_value(stream);

        match key_name
        {
            "title" => ...,
            "composer" => ...,
            "tempo" => ...,
            "beats" => ...,
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;


    fn parsetest(tokens: Vec<Token>, expected: PieceNode)
    {
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, 0) })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens);
        assert_eq!(result.pieces.len(), 1);
        assert_eq!(result.pieces[0], expected);
    }

    fn multiparsetest(tokens: Vec<Token>, expected: Vec<PieceNode>)
    {
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, 0) })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens);
        assert_eq!(result.pieces, expected);
    }

    #[test]
    fn parse_empty_file()
    {
        parsetest(vec![], PieceNode::default());
    }

    #[test]
    fn parse_empty_piece()
    {
        parsetest(vec![Piece, LeftBrace, RightBrace], PieceNode::default());
    }

    #[test]
    #[should_panic]
    fn parse_empty_piece_with_trailing_tokens_fails()
    {
        parsetest(vec![Piece, LeftBrace, RightBrace, RightBrace], PieceNode::default());
    }

    #[test]
    fn parse_multiple_empty_pieces()
    {
        multiparsetest(
            vec![Piece, LeftBrace, RightBrace, Piece, LeftBrace, RightBrace],
            vec![PieceNode::default(), PieceNode::default()]);
    }
}
