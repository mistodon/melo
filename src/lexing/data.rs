use error::SourceLoc;


#[derive(Debug, PartialEq, Eq)]
pub struct MetaToken<'a>
{
    pub token: Token<'a>,
    pub span: Span<'a>,
    pub loc: SourceLoc,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span<'a>(pub usize, pub &'a str);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token<'a>
{
    Piece,
    Voice,
    Section,
    Part,
    Play,
    LeftBrace,
    RightBrace,
    Comma,
    BlankLine,
    Num(i64),
    Key(&'a str),
    Ident(&'a str),
    Str(&'a str),

    Barline,
    Rest,
    Hit,
    Ditto,
    RepeatBar,
    ExtendNote,
    Note(&'a str),
    PlayPart(&'a str),

    EOF,
}

impl<'a> Token<'a>
{
    pub fn readable_type(&self) -> &'static str
    {
        use self::Token::*;

        match *self
        {
            Piece => "'piece'",
            Voice => "'voice'",
            Section => "'section'",
            Part => "'part'",
            Play => "'play'",
            LeftBrace => "'{'",
            RightBrace => "'}'",
            Comma => "','",
            BlankLine => "<blank_line>",
            Num(_) => "<number>",
            Key(_) => "<key>:",
            Ident(_) => "<identifier>",
            Str(_) => "<string>",
            Barline => "'|'",
            Rest => "'-'",
            Hit => "'x'",
            Ditto => "'\"'",
            RepeatBar => "'%'",
            ExtendNote => ".",
            Note(_) => "<note>",
            PlayPart(_) => "'*<part>'",
            EOF => "<end_of_file>",
        }
    }
}
