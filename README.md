# Melo

Melo is a music notation language and a compiler to MIDI. The goal is for it to be simple, readable, and expressive. The language is in its very early stages and at the moment only has a minimal set of features.

```
title: Simple Chords and Drums
tempo: 144

voice Piano { program: 4 }
voice Drums { drums }

play Piano
{
    :| G  a | a  b | c |
    :| E  E | F  G | G |
    :| C  C | C  D | E |
}

play Drums
{
    c#: | - | - | x |
    F#: | xx xx xx xx | xx xx xx xx | - |
    D:  | -- x- -- x- | -- x- -- x- | x |
    C:  | x- -- x- -- | x- -- x- -- | x |
}
```


## Installing

Currently, this project requires `cargo`:

`cargo install melo`

You could also clone the repo and build it. (Tested on Rust 1.23.0.)

It's also worth noting that I've only currently tested it on macOS.


## Usage

Easiest way to get started is to play one of the [examples](https://github.com/Pirh/melo/tree/master/pieces). If you cloned the repo, you can just run:

`melo mid pieces/rondo_alla_turca.melo --output rondo_alla_turca.mid`

This will generate a MIDI file from [this example](https://github.com/Pirh/melo/blob/master/pieces/rondo_alla_turca.melo).

If you have `timidity` installed, or if you set the `MELO_MIDI_PLAYER` environment variable to a command that can play MIDI files, then you can simply run:

`melo play pieces/rondo_alla_turca.melo`


## The language

### Top-level attributes

There are several top-level attributes that apply to an entire piece. You can set them as follows.

```
// Lines beginning with `//` are comments.

title: The Title of the Piece       // Spaces are allowed.
composer: Your Name                 // Same as above.
beats: 3                            // The number of beats per bar.
tempo: 120                          // The tempo of the piece in beats-per-minute.
```

Commas are optional when splitting attributes across multiple lines, but you can also do this:

```
title: A, composer: B, beats: 3
```

### Voices

Before you can play any notes, you need instruments to play them with. A voice is declared like this:

```
voice Piano         // An identifier for the voice.
{
    program: 4      // The MIDI program number for the instrument.
                    // Run `melo ref instruments` to see them all.

    channel: 1      // The MIDI channel this voice should play on. Defaults to `1`.
    octave: -1      // This can be used to offset notes by a number of octaves.
    volume: 127     // The volume of the voice, between 0 and 127.
}
```

There is also a special `drums` attribute which sets up some sensible defaults for a percussion voice:

```
voice Drums { drums }       // This is equivalent to the following voice:

voice Drums2 { program: 0, channel: 10, octave: -2 }
```

### Playing notes

In order to play notes with a given voice, you need to write a `play` block for that instrument. If you have multiple play blocks for different instruments, they will play simultaneously.

```
play Piano
{
    // The two staves below will play simultaneously.
    :| C D E F G a b c |        // There are 8 notes in this bar, so the notes are half as long...
    :| C,, E,, G,, C,  |        // As the 4 notes in this bar.

    // You can leave a blank line to continue on from the previous staves.
    :| b a G F E D C . | . |
    :| G,, E,, C,, .   | . |    // A `.` extends the length of the previous note.
}
```

The two staves used above began with `:`, meaning they had no `prefix`. If you are writing a drum part however, the prefix determines what note will be played on that stave.

```
play Drums
{
    // The `x` means hit the note, the `-` is a rest.
    F#: | xxx xxx xxx xxx |     // F# = Hi-hat
    D:  | --- x-- --- x-- |     // D = Snare drum
    C:  | x-x --x x-x --- |     // C = Kick drum, see `melo ref notes` for more information.
}
```

There may be other stave types added in future to support other properties of the music. For example, note velocity, accents, etc.


### Notes

The note `C` is middle C, as defined by the MIDI standard. The note below that is `B` and the (diatonic) note above that is `D`.

The note an octave higher is `c` and the note an octave above that is `c'`, then `c''` and so on.

The note an octave lower is `C,` then `C,,` and so on.

You can sharpen and flatten a note by using `C#` and `C_` respectively. However, bear in mind that - unlike traditional music notation, and ABC notation - these accidentals do not last for the entire bar. If you write: `:| C# C |`, then the first note is C sharp, and the second is C natural.

Another potential gotcha if you are used to ABC notation is that the octave boundaries are at the `A` notes:

```
Melo:   A  B  C D E F G a b c
ABC:    A, B, C D E F G A B c
```


## Use with vim

If you copy the files in [this directory](https://github.com/Pirh/melo/tree/master/vim) into your `.vim` directory - or vim runtime directory - you can get syntax highlighting and filetype detection for melo.


## TODO

### Future features

1.  Repeats of bars/sections
2.  Key signatures
3.  Dynamics
4.  Support pitch bends / panning / other MIDI features
5.  Explicit tuplets
6.  Changing attributes (tempo, volume, ...) during piece

### Future fixes

1.  Warnings/errors about missed bars/staves

