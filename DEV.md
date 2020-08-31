Notes
===

- Parts in the same scope play together
- Sections in the same scope play in sequence
- Piece -> Section -> Part -> Staves; each is implicit if not written
- If a section contains sections, only those inner sections will play
- If a section contains parts, but no section, the parts are played
- A section containing `$:` staves must contain nothing else
    - I'm unsure on this notation for reusing sections
- (Question: what if we want to be able to parameterize parts/sections? Think about it.)
- Names do not have to be scoped unless they would otherwise be ambiguous.
    - So no silent shadowing,

## Evolution of a piece

### 1. Doodling:

```
:| A C E G | A - E - | A
```

### 2. Adding attributes

```
tempo: 130
program: flute
volume: 0.5

:| A C E G | A - E - | A
```

### 3. Simultaneous parts

```
tempo: 130

part {
    program: flute
    volume: 0.5

    :| A C E G | A - E - | A
}

part {
    program: bass

    :| A A A . | % | A
}
```

### 4. Sequential sections

```
tempo: 130

section {
    part {
        program: flute
        volume: 0.5

        :| A C E G | A - E - |
    }

    part {
        program: bass

        :| A A A . | % |
    }
}

section {
    part {
        program: flute
        volume: 0.5

        :| A . . A | C E G . | G a G . | E
    }

    part {
        program: bass

        :| A A A . | % | % | % |
    }
}
```

### 5. Refactoring voices

```
tempo: 130

voice Flute {
    program: flute
    volume: 0.5
}

section {
    part {
        voice: Flute

        :| A C E G | A - E - |
    }

    part {
        program: bass

        :| A A A . | % |
    }
}

section {
    part {
        voice: Flute

        :| A . . A | C E G . | G a G . | E
    }

    part {
        program: bass

        :| A A A . | % | E E E . | %
    }
}
```

### 6. Refactoring parts

```
tempo: 130

voice Flute {
    program: flute
    volume: 0.5
}

section {
    part {
        voice: Flute

        :| A C E G | A - E - |
    }

    part BassA {
        program: bass

        :| A A A . | % |
    }
}

section {
    part {
        voice: Flute

        :| A . . A | C E G . | G a G . | E
    }

    part {
        %:| BassA |

        :| E E E . | %          // This plays after, as normal
    }
}
```

### 7. Reusing sections

```
tempo: 130

voice Flute {
    program: flute
    volume: 0.5
}

section SectionA {
    part {
        voice: Flute

        :| A C E G | A - E - |
    }

    part BassA {
        program: bass

        :| A A A . | % |
    }
}

section {
    part {
        voice: Flute

        :| A . . A | C E G . | G a G . | E
    }

    part {
        %:| SectionA.BassA |       // Optionally scoped names?

        :| E E E . | %
    }
}

section {
    $:| SectionA |
}
```

