// Whole file is in an implicit `piece {}` block. It is very similar to a `section`, and must contain at least one play block.
title: blah
composer: blah

// A voice defines an instrument, available in the current scope
voice Drums
{
    program: 0,
}

voice Guitar
{
    program: 29,
    channel: 1,
}

// A section contains one or more `play {}` blocks
section MainTheme
{
    // A play block is an instruction to play some notes - it takes a voice to play with, and some notes etc.
    play Drums
    {
        // A part is a repeatable bit of music. The `%` symbol is there to repeat the previous bar on the current stave.
        part Rhythm
        {
            F^  : x x x x | %. |
            D   : - x - - | - x - x | % |
            C   : x - x x | x - x - | % |
        }

        // You can "dereference" parts to play them. The `"` ditto symbol is there to copy the bar from the above stave.
        F^  : *Rhythm   | x x x x       | *Rhythm   | x x x x  |
        D   : "         | - - - xxxx    | "         | -x---x-- |
        C   : "         | x - x -       | "         | x-x-x--- |
    }

    play Guitar
    {
        part Scale
        {
            : cdefga'b'c' | b'a'gfedc. |
        }

        // The `:` stave prefix represents all staves. It will only accept parts, sections, and repeat symbols. The contents will be combined with other staves if present.
        :: *Scale | %.3 |
        // : CCCC | %.7 |     // If we uncommented this, it would play that C as a pedal over the `Scale` section, using the same voice.
        // As a side note, we need a concise but clear syntax for "repeat the previous N things, M times". Currently thinking `N%.M` with abbreviations: %.M == 1%.M, % == 1%.1, N%. == N%.2
    }
}

// A play block without instruments can only use the `::` stave type, described above.
play
{
    :: *MainTheme | % |
}

/*
Some other things we are missing!
===

1.  Some kind of `settings {}` block within sections, parts, and plays that allows changing the tempo, time signature, key, etc.
2.  An `info {}` block with arbitrary keys for adding metadata
3.  Ways to control the velocity/panning of notes etc.
*/

