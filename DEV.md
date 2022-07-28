Tasks
===

- [ ] Parser rewrite (yeah... it'll take a while...)
- [ ] Multi-bar repeats
- [ ] Repeat-n-times
- [ ] Repeatable sections

Syntax design
===

## Common structures:

```melo
// A file can just contain the inner part of a piece, making it the default one
piece Id {
    attribute_1: value
    attribute_2: value
    attribute_3: value ; attribute_4: value

    voice VoiceId {
        ...
    }

    voice {
        ...
    }

    part PartId {
        ...
    }

    section SectionId {
        ...
    }

    piece SubPieceId {
        ...
    }

    play VoiceId {
        ...
    }

    play {
        ...
    }

    // Implicit play block
    stave_1: stave
    stave_2: stave
}

// Identical to `piece` - it's just a non-public
section Id {
    ...
}

voice Id {
    attribute_1: value
    attribute_2: value
    attribute_3: value ; attribute_4: value
}

// You can also configure the default voice if you want:
// Voiceless play blocks will use this
voice {
    attribute_1: value
    attribute_2: value
    attribute_3: value ; attribute_4: value
}

part Id {
    attribute_1: value
    attribute_2: value
    attribute_3: value ; attribute_4: value

    stave_1: stave
    stave_2: stave

    part SubPartId {
        ...
    }
}

// This corresponds to notes on an actual MIDI channel
play VoiceId {
    attribute_1: value
    attribute_2: value
    attribute_3: value ; attribute_4: value

    stave_1: stave
    stave_2: stave
}
```

## Sections and multi-staves:

```melo
// Only part/section/piece blocks can contain play blocks (implicit or otherwise)
// The play blocks are always played in parallel
// To play them in sequence instead, use sequential sections
// The special multi-stave can be used to do this:
// Note that the multi-stave is not valid in `part`s, only section/piece
play {
    ::| SectionA | SectionB |
}

// The voiceless play block can also refer to the default voice
play {
    :| a b c d |
}

// But you can't mix them - this is invalid:
play {
    :| a b c d |
    ::| SectionA |
}

// To accomplish what you actually wanted there, use this. (The multi-stave
// play block is an implicit one here but doesn't have to be.)
// Remember all play blocks within a section-oid block are parallel.
section Blah {
    play {
        :| a b c d |
    }

    ::| SectionA |
}

// If you want to play multiple sections in parallel, you can:
play {
    ::| SectionA_High |
    ::| SectionA_Low |
}

// By default, you'll get a warning if you're trying to play parts, or sections
// or blocks of repeats in parallel when they're different lengths.
// Maybe even parallel play blocks that are different lengths?
// In any case, there's a token you can use to absolve yourself of having to
// care:
play {
    // The `~` explicitly pads ShortSection with silence
    ::| ShortSection | ~
    ::| LongSection |
}

// If you want to play the same voice twice in parallel, you'll have to give
// the play block a label:
section {
    play Violin {
        :| A C E
    }

    // This second play block would give an error (or at least warning?)
    play Violin {
        :| C E G
    }

    // Instead do this. The label has to be unique among voices in scope.
    play Violin as Violin2 {
        :| C E G
    }
}

// You can also play a single voice from a section, even if that's weird:
play Piano {
    // Only plays the Piano part from the main theme
    ::| Section_MainTheme |
}

// If you need to disambiguate, you can _choose_ the voice to pull:
play Violin {
    ::| Section_MainTheme.Violin2 |
}

// Even if it's not the intended voice:
play Violin {
    ::| Section_MainTheme.Piano |
}
```

## Parts
```melo
// To play an individual part, do this:
play {
    p:| PartId
}

// You can mix it with other staves:
play {
    // This is the first "part":
    :| a b c d
    :| e f# g a'

    // The second part is played after
    p:| PartId

    // Third parts etc.
    :| a b a b
}

// You can play a subset of staves of a part too:
play {
    // This will play the first bar of the first stave of `PartId`
    // during the second bar here:
    :| a b c d | *PartId |
    :| e f# g a' | g f# e . |
}
```

## Attributes
```melo
// Most attributes are valid in any block, with vaguely different meanings:
piece {
    // Plays every single note in the piece/section an octave higher
    octave: 1
}

voice A {
    // When using this voice, every note is played an octave higher than normal
    octave: 1
}

part A {
    // Notes in this part are stored an octave higher than they are written
    octave: 1
}

play A {
    // Anything played in this block is an octave higher
    octave: 1
}

// Some attributes are only valid in piece/section blocks:
piece {
    title: ...
    composer: ...
    beats: ...
    tempo: ...
}

// This can get messy:
piece {
    // This will just use the beats/tempo from the first (FastSection)
    // And also log a warning, because it'll probably sound bad.
    ::| FastSection
    ::| SlowSection
}
```

## Stave types
```melo
// Note stave(s)
:| a b c d

:| a b c d
:| c d e f

// Single note (percussion) stave(s)
F#:| x x - x |
B: | x - - - |

// Multi-stave
::| SectionName |

// Part stave
p:| PartName |

// Every-stave (puts the same value in every previously defined stave)
 :| a b c d
 :| c d e f
B:| x x - -

@:| % | // Repeat last bar for all staves
@:| - | - | - | - | // Rest for four bars on all staves

@:| *PartName | // Equivalent to p:| PartName |

// For later - microtone staves (how to do numerical durations?)
e100:| e0 e70 e0 e50 e70 e0' . . |
```

## Confusing notations

```melo
// You can define parts in any scope, and you can even define sections inside voiceless play blocks
piece {
    part A {
    }

    section Section2 {
        part B {
        }
    }

    play A {
        part C {
        }
    }

    play {
        part D {
        }

        section Section3 {
        }
    }
}

// This means you can bisect a play block with a part
// The notes are the same as if you had omitted the part entirely
play {
    :| a b c d

    part B {
    }

    :| e f g a'
}

// The same occurs if you do this in a section
section A {
    :| a b c d

    part B {
    }

    :| e f g a'
}
// Because this section became one big implicit play block after the first stave

// However you cannot nest play blocks, so while this is valid:
section A {
    play Piano {
        :| a b c
    }

    :| d e f
}

// This is NOT valid:
section A {
    :| d e f

    play Piano {
        :| a b c
    }
}

// Basically, implicit blocks consume all syntax up to the end of the explicit
// block they were defined within.
// This has other awkward implications:

// This works
section A {
    part B {
    }

    p:| B |
}

// This works, because parts can be defined later in scope
section A {
    play {
        p:| B |
    }

    part B {
    }
}

// This works, because part B is now defined within the implicit play block
section A {
    p:| B |

    part B {
    }
}

// But this does NOT work, because it's parsed in a way that B is nested deeper
// than the explicit play block.
section A {
    play {
        p:| B |
    }

    :| a b c

    part B {
    }
}

// It's parsed like this:
section A {
    play {
        p:| B |
    }

    play {
        :| a b c

        part B {
        }
    }
}
// See? part B is not in scope for the first play block
```

## Cute quirky ideas
- What if `...` is a valid token, equivalent to `todo!()` in rust?
- You can use `;` to separate attributes and staves instead of newlines
- You can use `;;` which is equivalent except it also breaks a grand stave
- Repeats:
    - `%` repeat last bar
    - `%%` repeat last two bars
    - `%(n)` repeat last `n` bars
    - `%2` repeat last bar twice
    - `%%2` repeat last two bars twice
    - `%(n)k` repeat last `n` bars `k` times
- `e10:` stave:
    - Splits an octave into `10` parts with the notes `e0` to `e9`.
    - Any number of splits is allowed.
    - How do we specify the root?
    - Also why `e`? Where did we see that?
- Meta attributes? Maybe for things like `drums` that feel a bit messy?
-
    ```
    #[drums]
    voice Drums {
        ...
    }
    ```
- Empty blocks? `voice Drums;`

## Implementation details

### Parsing contexts
- Block-level:
    - We're looking for attributes, staves, or nested blocks
- Attribute value:
    - Depending on the attribute name, we accept different types. e.g:
        - String for `title`
        - Integer for `beats`
        - Integer-or-float for `tempo`
- Stave value (when we see a `|` after a `:`):
    - Depending on the stave type, we enter different parsing modes
        - Note-mode: we look for notes, rests, etc. Part/section names must be preceeded by `*`
            - Used by `:|`, `<note>:|`, `@:|`
        - Chunk-mode: we look for names (allowing rests, repeats as well)
            - Used by `::|` and `p:|`

