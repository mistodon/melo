pub mod data;
pub mod error;


use std::borrow::Cow;
use std::iter::Peekable;
use std::slice::Iter;

use lexing::data::*;
use lexing::data::Token::*;
use notes;
use trust::Trust;


use self::data::*;
use self::error::{ErrorType, ParsingError};


type TokenStream<'a> = Peekable<Iter<'a, MetaToken<'a>>>;


fn error_swizzle<T, E>(results: Vec<Result<T, E>>) -> Result<Vec<T>, Vec<E>>
where
    T: ::std::fmt::Debug,
    E: ::std::fmt::Debug,
{
    let any_errors = results.iter().any(Result::is_err);

    if any_errors
    {
        Err(results.into_iter().filter_map(Result::err).collect())
    }
    else
    {
        Ok(results.into_iter().map(Result::unwrap).collect())
    }
}


pub fn parse<'a>(tokens: &'a [MetaToken<'a>]) -> Result<ParseTree<'a>, ParsingError>
{
    assert_eq!(
        tokens.last().map(|meta| meta.token),
        Some(EOF),
        "Missing EOF from token stream."
    );

    let mut stream = tokens.iter().peekable();

    let pieces = match stream.peek().trust().token
    {
        Piece =>
        {
            let mut piece_results = Vec::new();

            while stream.peek().trust().token != EOF
            {
                piece_results.push(parse_piece(&mut stream));
            }

            error_swizzle(piece_results)?
        }
        _ => vec![parse_piece_from_body(&mut stream)?],
    };

    match stream.next().trust()
    {
        meta if meta.token == EOF => Ok(ParseTree { pieces }),
        meta => Err(ParsingError::unexpected(
            meta,
            "after `piece`",
            "end of file".to_owned(),
        )),
    }
}


fn expect_token(
    stream: &mut TokenStream,
    token: Token,
    context: &'static str,
) -> Result<(), ParsingError>
{
    let meta = *stream.peek().trust();

    let result = match meta.token
    {
        EOF =>
        {
            return Err(ParsingError::eof(
                meta,
                context,
                token.readable_type().to_owned(),
            ))
        }
        found if found == token => Ok(()),
        _ => Err(ParsingError::unexpected(
            meta,
            context,
            token.readable_type().to_owned(),
        )),
    };

    stream.next();

    result
}

fn skip_token(stream: &mut TokenStream, token: Token) -> bool
{
    if stream.peek().trust().token == token
    {
        stream.next();
        true
    }
    else
    {
        false
    }
}

fn poison_scope(stream: &mut TokenStream, open_delim: Token, close_delim: Token)
{
    let mut nest = 1;

    while nest > 0
    {
        match stream.peek().trust().token
        {
            EOF => break,
            t if t == open_delim => nest += 1,
            t if t == close_delim => nest -= 1,
            _ => (),
        }

        stream.next();
    }
}


fn parse_piece<'a>(stream: &mut TokenStream<'a>) -> Result<PieceNode<'a>, ParsingError>
{
    expect_token(stream, Piece, "in top-level of file")?;
    expect_token(stream, LeftBrace, "at `piece`")?;

    let piece_node = parse_piece_from_body(stream);
    if piece_node.is_err()
    {
        poison_scope(stream, LeftBrace, RightBrace);
    }
    let piece_node = piece_node?;

    expect_token(stream, RightBrace, "after `piece`")?;

    Ok(piece_node)
}

fn parse_piece_from_body<'a>(
    stream: &mut TokenStream<'a>,
) -> Result<PieceNode<'a>, ParsingError>
{
    let mut voice_results = Vec::new();
    let mut play_results = Vec::new();
    let mut title = None;
    let mut composer = None;
    let mut beats = None;
    let mut tempo = None;

    loop
    {
        let meta = *stream.peek().trust();
        match meta.token
        {
            EOF | RightBrace => break,
            BlankLine =>
            {
                stream.next();
            }
            Voice =>
            {
                let voice = parse_voice(stream);
                if voice.is_err()
                {
                    poison_scope(stream, LeftBrace, RightBrace);
                }
                voice_results.push(voice);
            }
            Play =>
            {
                let play = parse_play(stream);
                if play.is_err()
                {
                    poison_scope(stream, LeftBrace, RightBrace);
                }
                play_results.push(play);
            }
            _ =>
            {
                let attribute_key = parse_attribute_key(stream, "in `piece`")?;
                match attribute_key
                {
                    Key("title") => title = Some(try_parse_name(stream, "after `title:`")?),
                    Key("composer") =>
                    {
                        composer = Some(try_parse_name(stream, "after `composer:`")?)
                    }
                    Key("tempo") =>
                    {
                        tempo = Some(try_parse_num(stream, "after `tempo:`")? as u64)
                    }
                    Key("beats") =>
                    {
                        beats = Some(try_parse_num(stream, "after `beats:`")? as u64)
                    }
                    Key(key) | Ident(key) =>
                    {
                        return Err(ParsingError {
                            loc: meta.loc.clone(),
                            error: ErrorType::InvalidAttribute {
                                attribute: key.to_owned(),
                                structure: "piece",
                            },
                        })
                    }
                    _ => unreachable!(),
                }

                let keep_finding_attributes = skip_token(stream, Comma);
                if !keep_finding_attributes
                {
                    break
                }
            }
        }
    }

    let voices = error_swizzle(voice_results)?;
    let plays = error_swizzle(play_results)?;

    Ok(PieceNode {
        title,
        composer,
        beats,
        tempo,
        voices,
        plays,
    })
}

fn parse_attribute_key<'a>(
    stream: &mut TokenStream<'a>,
    context: &'static str,
) -> Result<Token<'a>, ParsingError>
{
    let meta = stream.next().trust();
    match meta.token
    {
        EOF => Err(ParsingError::eof(
            meta,
            context,
            format!("{}", "an attribute key".to_owned()),
        )),
        Key(_) | Ident(_) => Ok(meta.token),
        _ => Err(ParsingError::unexpected(
            meta,
            context,
            "an attribute key".to_owned(),
        )),
    }
}

fn try_parse_name<'a>(
    stream: &mut TokenStream<'a>,
    context: &'static str,
) -> Result<&'a str, ParsingError>
{
    let meta = *stream.peek().trust();

    match meta.token
    {
        EOF => Err(ParsingError::eof(meta, context, "a name".to_owned())),
        Ident(s) | Str(s) =>
        {
            stream.next();
            Ok(s)
        }
        _ => Err(ParsingError::unexpected(meta, context, "a name".to_owned())),
    }
}

fn try_parse_num(stream: &mut TokenStream, context: &'static str)
    -> Result<i64, ParsingError>
{
    let meta = *stream.peek().trust();

    match meta.token
    {
        EOF => Err(ParsingError::eof(meta, context, "a number".to_owned())),
        Num(n) =>
        {
            stream.next();
            Ok(n)
        }
        _ => Err(ParsingError::unexpected(
            meta,
            context,
            "a number".to_owned(),
        )),
    }
}

fn parse_voice<'a>(stream: &mut TokenStream<'a>) -> Result<VoiceNode<'a>, ParsingError>
{
    expect_token(stream, Voice, "in `piece`")?;

    let name = try_parse_name(stream, "in `voice`")?;
    let mut channel = None;
    let mut program = None;
    let mut octave = None;
    let mut volume = None;

    expect_token(stream, LeftBrace, "at `voice`")?;

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        let meta = *stream.peek().trust();

        let attribute_key = parse_attribute_key(stream, "in `voice`")?;

        match attribute_key
        {
            Ident("drums") =>
            {
                channel = Some(10);
                octave = Some(-2);
            }
            Key("channel") => channel = Some(try_parse_num(stream, "after `channel:`")? as u8),
            Key("program") => program = Some(try_parse_num(stream, "after `program:`")? as u8),
            Key("octave") => octave = Some(try_parse_num(stream, "after `octave:`")? as i8),
            Key("volume") => volume = Some(try_parse_num(stream, "after `volume:`")? as u8),
            Key(key) | Ident(key) =>
            {
                return Err(ParsingError {
                    loc: meta.loc.clone(),
                    error: ErrorType::InvalidAttribute {
                        attribute: key.to_owned(),
                        structure: "voice",
                    },
                })
            }
            _ => unreachable!(),
        }

        if !skip_token(stream, Comma)
        {
            expect_token(stream, RightBrace, "after `voice`")?;
            break
        }
    }

    Ok(VoiceNode {
        name,
        channel,
        program,
        octave,
        volume,
    })
}

fn parse_play<'a>(stream: &mut TokenStream<'a>) -> Result<PlayNode<'a>, ParsingError>
{
    expect_token(stream, Play, "in `piece`")?;

    let error_loc = Some(stream.peek().trust().loc.clone());

    let voice = try_parse_name(stream, "in `play`").ok();
    let mut staves: Vec<StaveNode> = Vec::new();

    let mut anonymous_stave_count = 0;
    let mut allow_new_staves = true;

    expect_token(stream, LeftBrace, "at `play`")?;

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        let meta = stream.next().trust();

        match meta.token
        {
            EOF =>
            {
                return Err(ParsingError::eof(
                    meta,
                    "in `play`",
                    "a stave prefix".to_owned(),
                ))
            }
            BlankLine =>
            {
                let already_have_some_staves = !staves.is_empty();
                if already_have_some_staves
                {
                    allow_new_staves = false;
                    anonymous_stave_count = 0;
                }
            }
            Key(raw_prefix) =>
            {
                expect_token(stream, Barline, "after stave prefix")?;

                let prefix = match raw_prefix
                {
                    "" =>
                    {
                        let anonymous_prefix = format!("V{}", anonymous_stave_count);
                        anonymous_stave_count += 1;
                        Cow::Owned(anonymous_prefix)
                    }
                    prefix => Cow::Borrowed(prefix),
                };

                let stave = {
                    let existing_stave = staves
                        .iter()
                        .enumerate()
                        .find(|&(_, stave)| stave.prefix == prefix)
                        .map(|(i, _)| i);

                    let stave_index = existing_stave.unwrap_or_else(|| staves.len());

                    if existing_stave.is_none()
                    {
                        if allow_new_staves
                        {
                            staves.push(StaveNode {
                                prefix,
                                bars: Vec::new(),
                            });
                        }
                        else
                        {
                            return Err(ParsingError {
                                loc: meta.loc.clone(),
                                error: ErrorType::UndeclaredStave {
                                    stave_prefix: raw_prefix.to_owned(),
                                },
                            })
                        }
                    }

                    &mut staves[stave_index]
                };


                let stave_note = notes::note_to_midi(&stave.prefix);
                let mut bar = BarNode::default();

                loop
                {
                    let mut bar_full = false;
                    let mut stave_full = false;

                    let meta = *stream.peek().trust();

                    match meta.token
                    {
                        EOF =>
                        {
                            return Err(ParsingError::eof(
                                meta,
                                "in stave",
                                "stave contents".to_owned(),
                            ))
                        }
                        Rest =>
                        {
                            bar.notes.push(NoteNode::Rest { length: 1 });
                            bar.note_locs.push(meta.loc.clone());
                        }
                        Hit =>
                        {
                            let midi = stave_note.ok_or_else(|| ParsingError {
                                loc: meta.loc.clone(),
                                error: ErrorType::InvalidHit {
                                    stave_prefix: raw_prefix.to_owned(),
                                },
                            })?;
                            bar.notes.push(NoteNode::Note { midi, length: 1 });
                            bar.note_locs.push(meta.loc.clone());
                        }
                        Note(note) =>
                        {
                            let midi =
                                notes::note_to_midi(note).ok_or_else(|| ParsingError {
                                    loc: meta.loc.clone(),
                                    error: ErrorType::InvalidNote {
                                        note: note.to_owned(),
                                    },
                                })?;
                            bar.notes.push(NoteNode::Note { midi, length: 1 });
                            bar.note_locs.push(meta.loc.clone());
                        }
                        ExtendNote =>
                        {
                            bar.notes.push(NoteNode::Extension { length: 1 });
                            bar.note_locs.push(meta.loc.clone());
                        }
                        Num(num) =>
                        {
                            if num <= 0 || num >= 255
                            {
                                return Err(ParsingError {
                                    loc: meta.loc.clone(),
                                    error: ErrorType::InvalidLength { length: num },
                                })
                            }

                            let previous_note = bar.notes.last_mut().ok_or(ParsingError {
                                loc: meta.loc.clone(),
                                error: ErrorType::UnexpectedLength { length: num },
                            })?;

                            match *previous_note
                            {
                                NoteNode::Rest { ref mut length }
                                | NoteNode::Extension { ref mut length }
                                | NoteNode::Note { ref mut length, .. } => *length = num as u8,
                            }
                        }
                        Barline => bar_full = true,
                        Key(_) | BlankLine | RightBrace => stave_full = true,
                        _ =>
                        {
                            return Err(ParsingError::unexpected(
                                meta,
                                "in stave",
                                "stave contents".to_owned(),
                            ))
                        }
                    }

                    if (bar_full || stave_full) && !bar.notes.is_empty()
                    {
                        let complete_bar = ::std::mem::replace(&mut bar, BarNode::default());
                        stave.bars.push(complete_bar);
                    }

                    if stave_full
                    {
                        break
                    }

                    stream.next();
                }
            }
            _ =>
            {
                return Err(ParsingError::unexpected(
                    meta,
                    "in `play`",
                    "a stave prefix".to_owned(),
                ))
            }
        }
    }

    Ok(PlayNode {
        voice,
        staves,
        error_loc,
    })
}


#[cfg(test)]
mod tests
{
    use super::*;
    use test_helpers::stave;


    // TODO(claire): Clearly need a better way of carriaging errors
    fn doctor(parse_tree: &mut ParseTree)
    {
        for piece in &mut parse_tree.pieces
        {
            for play in &mut piece.plays
            {
                play.error_loc = None;

                for stave in &mut play.staves
                {
                    for bar in &mut stave.bars
                    {
                        bar.note_locs.clear();
                    }
                }
            }
        }
    }

    fn parsetest(source: &str, expected: PieceNode)
    {
        use lexing;

        let tokens = lexing::lex(source, None).unwrap();
        let mut result = parse(&tokens).unwrap();

        doctor(&mut result);

        assert_eq!(result.pieces.len(), 1);
        assert_eq!(result.pieces[0], expected);
    }

    fn parsefailtest(source: &str)
    {
        use lexing;

        let tokens = lexing::lex(source, None).unwrap();
        assert!(parse(&tokens).is_err());
    }

    fn multiparsetest(source: &str, expected: Vec<PieceNode>)
    {
        use lexing;

        let tokens = lexing::lex(source, None).unwrap();
        let mut result = parse(&tokens).unwrap();

        doctor(&mut result);

        assert_eq!(result.pieces, expected);
    }


    #[test]
    fn parse_empty_file()
    {
        parsetest("", PieceNode::default());
    }

    #[test]
    fn parse_empty_piece()
    {
        parsetest("piece {}", PieceNode::default());
    }

    #[test]
    #[should_panic]
    fn parse_empty_piece_with_trailing_tokens_fails()
    {
        parsetest("piece {}}", PieceNode::default());
    }

    #[test]
    fn parse_multiple_empty_pieces()
    {
        multiparsetest(
            "piece {} piece {}",
            vec![PieceNode::default(), PieceNode::default()],
        );
    }

    #[test]
    fn parse_attributes_in_piece()
    {
        parsetest(
            "title: \"The Title of the Piece\", composer: Claire, beats: 5, tempo: 123",
            PieceNode {
                title: Some("The Title of the Piece"),
                composer: Some("Claire"),
                beats: Some(5),
                tempo: Some(123),
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_attributes_with_trailing_comma()
    {
        parsetest(
            "title: Title,",
            PieceNode {
                title: Some("Title"),
                ..Default::default()
            },
        );
    }

    #[test]
    fn fail_to_parse_invalid_attributes()
    {
        parsefailtest("titel: Title,");
    }

    #[test]
    fn parse_empty_voice()
    {
        parsetest(
            "voice Drums {}",
            PieceNode {
                voices: vec![
                    VoiceNode {
                        name: "Drums",
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
    }

    #[test]
    fn parse_voice_attributes()
    {
        parsetest(
            "voice Lead { channel: 1, program: 0, octave: -2, volume: 99, }",
            PieceNode {
                voices: vec![
                    VoiceNode {
                        name: "Lead",
                        channel: Some(1),
                        program: Some(0),
                        octave: Some(-2),
                        volume: Some(99),
                    },
                ],
                ..Default::default()
            },
        )
    }

    #[test]
    fn parse_valueless_attribute()
    {
        parsetest(
            "voice Drums { drums }",
            PieceNode {
                voices: vec![
                    VoiceNode {
                        name: "Drums",
                        channel: Some(10),
                        octave: Some(-2),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_empty_play_node()
    {
        parsetest(
            "play Drums {}",
            PieceNode {
                plays: vec![
                    PlayNode {
                        voice: Some("Drums"),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_empty_voiceless_play_node()
    {
        parsetest(
            "play {}",
            PieceNode {
                plays: vec![
                    PlayNode {
                        voice: None,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_play_node_with_stave()
    {
        parsetest(
            "play { :| - }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![stave("V0", vec![vec![NoteNode::Rest { length: 1 }]])],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_play_node_with_extra_barlines()
    {
        parsetest(
            "play { :|| - || }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![stave("V0", vec![vec![NoteNode::Rest { length: 1 }]])],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_play_node_with_two_staves()
    {
        parsetest(
            "play { C:| - ; D:| - }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave("C", vec![vec![NoteNode::Rest { length: 1 }]]),
                            stave("D", vec![vec![NoteNode::Rest { length: 1 }]]),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_play_node_with_percussive_notes()
    {
        parsetest(
            "play { C:| x }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "C",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_play_node_with_melody_notes()
    {
        parsetest(
            "play { :| C D }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "V0",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                        NoteNode::Note {
                                            midi: 62,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_stave_split_over_multiple_lines()
    {
        parsetest(
            "play { C:| x ; C:| x }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "C",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                    ],
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_multiple_concurrent_melody_lines()
    {
        parsetest(
            "play { :| C ; :| G }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "V0",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                            stave(
                                "V1",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 67,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn parse_multiple_concurrent_melody_lines_broken_up_by_blank_line()
    {
        parsetest(
            "play { :| C ; :| G\n\n:| G ; :| d }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "V0",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 1,
                                        },
                                    ],
                                    vec![
                                        NoteNode::Note {
                                            midi: 67,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                            stave(
                                "V1",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 67,
                                            length: 1,
                                        },
                                    ],
                                    vec![
                                        NoteNode::Note {
                                            midi: 74,
                                            length: 1,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        );
    }

    #[test]
    fn fail_parsing_too_many_staves_after_blank_line()
    {
        parsefailtest("play { :| C\n\n:| G ; :| d }");
    }

    #[test]
    fn fail_when_hit_notes_are_encountered_in_incompatible_staves()
    {
        parsefailtest("play { :| x }");
    }

    #[test]
    fn parse_note_with_length()
    {
        parsetest(
            "play { :| C4 }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![
                            stave(
                                "V0",
                                vec![
                                    vec![
                                        NoteNode::Note {
                                            midi: 60,
                                            length: 4,
                                        },
                                    ],
                                ],
                            ),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
    }

    #[test]
    fn parse_rest_with_length()
    {
        parsetest(
            "play { :| -4 }",
            PieceNode {
                plays: vec![
                    PlayNode {
                        staves: vec![stave("V0", vec![vec![NoteNode::Rest { length: 4 }]])],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
    }

    #[test]
    fn fail_on_unexpected_length()
    {
        parsefailtest("play { :| 4 }");
    }

    #[test]
    fn fail_on_overflowed_length()
    {
        parsefailtest("play { :| -300 }");
    }

    #[test]
    fn fail_on_underflowed_length()
    {
        parsefailtest("play { :| -0 }");
    }
}
