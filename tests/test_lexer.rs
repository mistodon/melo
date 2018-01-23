extern crate midscript;

#[macro_use]
extern crate pretty_assertions;


#[test]
fn test_lexing()
{
    use midscript::lexing::{ self, Token };
    use midscript::lexing::Token::*;

    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/lexing_test.midscript"));

    let result = lexing::lex(source).unwrap();

    let token_types = result.iter().map(|meta_token| meta_token.token).collect::<Vec<Token>>();

    let expected = vec![
        Key("title"), Str("Lexer Test"),
        Key("composer"), Str("Claire Harris"),
        BlankLine,

        Voice, Ident("Lead"),
        LeftBrace,
        Key("program"), Num(29), Comma,
        Key("channel"), Num(1), Comma,
        RightBrace,
        BlankLine,

        Voice, Ident("Drums"),
        LeftBrace,
        Ident("drums"), Comma,
        Key("octave"), Num(-2),
        RightBrace,
        BlankLine,

        Section, Ident("MainTheme"),
        LeftBrace,
        Play, Ident("Drums"),
        LeftBrace,
        Part, Ident("Rhythm0"),
        LeftBrace,

        Key("F^"), Barline,
        Hit, Hit, Hit, Hit, Hit, Hit, Hit, Hit, Barline,
        Repeat, Barline,
        Repeat, Barline,

        Key("D"), Barline,
        Rest, Rest, Hit, Rest, Rest, Rest, Hit, Rest, Barline,
        Rest, Rest, Hit, Rest, Rest, Rest, Hit, Rest, Barline,
        Rest, Hit, Rest, Hit, Barline,

        Key("C"), Barline,
        Hit, Rest, Rest, Rest, Hit, Rest, Rest, Rest, Barline,
        Hit, Rest, Rest, Rest, Hit, Hit, Rest, Rest, Barline,
        Hit, Rest, Hit, Rest, Barline,

        RightBrace,
        BlankLine,

        Key("F^"), Barline,
        PlayPart("Rhythm0"), Barline,
        Hit, Rest, Hit, Rest, Hit, Rest, Hit, Rest, Hit, Rest, Hit, Rest, Rest, Rest, Rest, Rest, Barline,
        PlayPart("Rhythm0"), Barline,
        Hit, Hit, Hit, Hit, Hit, Rest, Rest, Rest, Barline,

        Key("D"), Barline,
        Ditto, Barline,
        Rest, Rest, Rest, Rest, Hit, Rest, Rest, Rest, Rest, Rest, Rest, Rest, Hit, Hit, Hit, Hit, Barline,
        Ditto, Barline,
        Rest, Rest, Hit, Rest, Hit, Rest, Rest, Rest, Barline,

        Key("C"), Barline,
        Ditto, Barline,
        Hit, Rest, Rest, Rest, Rest, Rest, Rest, Rest, Hit, Rest, Hit, Rest, Rest, Rest, Rest, Rest, Barline,
        Ditto, Barline,
        Hit, Hit, Rest, Hit, Hit, Rest, Rest, Rest, Barline,

        RightBrace,
        BlankLine,

        Play, Ident("Guitar"),
        LeftBrace,

        Part, Ident("Melody0"),
        LeftBrace,
        Key(""), Barline,
        Note("C,"), Note("G,"), Barline,
        Note("C"), Rest, Barline,
        RightBrace,
        BlankLine,

        Key(""), Barline,
        PlayPart("Melody0"), Barline,
        Repeat, Barline, Repeat, Barline, Repeat, Barline,

        RightBrace,
        RightBrace,
        BlankLine,

        Play,
        LeftBrace,
        Key(":"), Barline,
        PlayPart("MainTheme"), Barline,
        Repeat, Barline,

        RightBrace,
        BlankLine,
        EOF
    ];

    assert_eq!(token_types, expected);
}
