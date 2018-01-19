const MIDSCRIPT_SHARPS: [&str; 128] = [
    "C,,,,,",
    "C^,,,,,",
    "D,,,,,",
    "D^,,,,,",
    "E,,,,,",
    "F,,,,,",
    "F^,,,,,",
    "G,,,,,",
    "G^,,,,,",
    "A,,,,",
    "A^,,,,",
    "B,,,,",
    "C,,,,",
    "C^,,,,",
    "D,,,,",
    "D^,,,,",
    "E,,,,",
    "F,,,,",
    "F^,,,,",
    "G,,,,",
    "G^,,,,",
    "A,,,",
    "A^,,,",
    "B,,,",
    "C,,,",
    "C^,,,",
    "D,,,",
    "D^,,,",
    "E,,,",
    "F,,,",
    "F^,,,",
    "G,,,",
    "G^,,,",
    "A,,",
    "A^,,",
    "B,,",
    "C,,",
    "C^,,",
    "D,,",
    "D^,,",
    "E,,",
    "F,,",
    "F^,,",
    "G,,",
    "G^,,",
    "A,",
    "A^,",
    "B,",
    "C,",
    "C^,",
    "D,",
    "D^,",
    "E,",
    "F,",
    "F^,",
    "G,",
    "G^,",
    "A",
    "A^",
    "B",
    "C",
    "C^",
    "D",
    "D^",
    "E",
    "F",
    "F^",
    "G",
    "G^",
    "a",
    "a^",
    "b",
    "c",
    "c^",
    "d",
    "d^",
    "e",
    "f",
    "f^",
    "g",
    "g^",
    "a'",
    "a^'",
    "b'",
    "c'",
    "c^'",
    "d'",
    "d^'",
    "e'",
    "f'",
    "f^'",
    "g'",
    "g^'",
    "a''",
    "a^''",
    "b''",
    "c''",
    "c^''",
    "d''",
    "d^''",
    "e''",
    "f''",
    "f^''",
    "g''",
    "g^''",
    "a'''",
    "a^'''",
    "b'''",
    "c'''",
    "c^'''",
    "d'''",
    "d^'''",
    "e'''",
    "f'''",
    "f^'''",
    "g'''",
    "g^'''",
    "a''''",
    "a^''''",
    "b''''",
    "c''''",
    "c^''''",
    "d''''",
    "d^''''",
    "e''''",
    "f''''",
    "f^''''",
    "g''''",
];

const MIDSCRIPT_FLATS: [&str; 128] = [
    "C,,,,,",
    "D_,,,,,",
    "D,,,,,",
    "E_,,,,,",
    "E,,,,,",
    "F,,,,,",
    "G_,,,,,",
    "G,,,,,",
    "A_,,,,",
    "A,,,,",
    "B_,,,,",
    "B,,,,",
    "C,,,,",
    "D_,,,,",
    "D,,,,",
    "E_,,,,",
    "E,,,,",
    "F,,,,",
    "G_,,,,",
    "G,,,,",
    "A_,,,",
    "A,,,",
    "B_,,,",
    "B,,,",
    "C,,,",
    "D_,,,",
    "D,,,",
    "E_,,,",
    "E,,,",
    "F,,,",
    "G_,,,",
    "G,,,",
    "A_,,",
    "A,,",
    "B_,,",
    "B,,",
    "C,,",
    "D_,,",
    "D,,",
    "E_,,",
    "E,,",
    "F,,",
    "G_,,",
    "G,,",
    "A_,",
    "A,",
    "B_,",
    "B,",
    "C,",
    "D_,",
    "D,",
    "E_,",
    "E,",
    "F,",
    "G_,",
    "G,",
    "A_",
    "A",
    "B_",
    "B",
    "C",
    "D_",
    "D",
    "E_",
    "E",
    "F",
    "G_",
    "G",
    "a_",
    "a",
    "b_",
    "b",
    "c",
    "d_",
    "d",
    "e_",
    "e",
    "f",
    "g_",
    "g",
    "a_'",
    "a'",
    "b_'",
    "b'",
    "c'",
    "d_'",
    "d'",
    "e_'",
    "e'",
    "f'",
    "g_'",
    "g'",
    "a_''",
    "a''",
    "b_''",
    "b''",
    "c''",
    "d_''",
    "d''",
    "e_''",
    "e''",
    "f''",
    "g_''",
    "g''",
    "a_'''",
    "a'''",
    "b_'''",
    "b'''",
    "c'''",
    "d_'''",
    "d'''",
    "e_'''",
    "e'''",
    "f'''",
    "g_'''",
    "g'''",
    "a_''''",
    "a''''",
    "b_''''",
    "b''''",
    "c''''",
    "d_''''",
    "d''''",
    "e_''''",
    "e''''",
    "f''''",
    "g_''''",
    "g''''",
];

const ABC_NOTES: [&str; 128] = [
    "C,,,,,",
    "^C,,,,,",
    "D,,,,,",
    "^D,,,,,",
    "E,,,,,",
    "F,,,,,",
    "^F,,,,,",
    "G,,,,,",
    "^G,,,,,",
    "A,,,,,",
    "^A,,,,,",
    "B,,,,,",
    "C,,,,",
    "^C,,,,",
    "D,,,,",
    "^D,,,,",
    "E,,,,",
    "F,,,,",
    "^F,,,,",
    "G,,,,",
    "^G,,,,",
    "A,,,,",
    "^A,,,,",
    "B,,,,",
    "C,,,",
    "^C,,,",
    "D,,,",
    "^D,,,",
    "E,,,",
    "F,,,",
    "^F,,,",
    "G,,,",
    "^G,,,",
    "A,,,",
    "^A,,,",
    "B,,,",
    "C,,",
    "^C,,",
    "D,,",
    "^D,,",
    "E,,",
    "F,,",
    "^F,,",
    "G,,",
    "^G,,",
    "A,,",
    "^A,,",
    "B,,",
    "C,",
    "^C,",
    "D,",
    "^D,",
    "E,",
    "F,",
    "^F,",
    "G,",
    "^G,",
    "A,",
    "^A,",
    "B,",
    "C",
    "^C",
    "D",
    "^D",
    "E",
    "F",
    "^F",
    "G",
    "^G",
    "A",
    "^A",
    "B",
    "c",
    "^c",
    "d",
    "^d",
    "e",
    "f",
    "^f",
    "g",
    "^g",
    "a",
    "^a",
    "b",
    "c'",
    "^c'",
    "d'",
    "^d'",
    "e'",
    "f'",
    "^f'",
    "g'",
    "^g'",
    "a'",
    "^a'",
    "b'",
    "c''",
    "^c''",
    "d''",
    "^d''",
    "e''",
    "f''",
    "^f''",
    "g''",
    "^g''",
    "a''",
    "^a''",
    "b''",
    "c'''",
    "^c'''",
    "d'''",
    "^d'''",
    "e'''",
    "f'''",
    "^f'''",
    "g'''",
    "^g'''",
    "a'''",
    "^a'''",
    "b'''",
    "c''''",
    "^c''''",
    "d''''",
    "^d''''",
    "e''''",
    "f''''",
    "^f''''",
    "g''''",
];


pub fn note_to_midi(note: &str) -> i8
{
    let mut chars = note.chars();
    let mut midi = match chars.next().unwrap()
    {
        'A' => 57,
        'B' => 59,
        'C' => 60,
        'D' => 62,
        'E' => 64,
        'F' => 65,
        'G' => 67,
        'a' => 69,
        'b' => 71,
        'c' => 72,
        'd' => 74,
        'e' => 76,
        'f' => 77,
        'g' => 79,
        c => panic!("No such note exists: '{}'", c)
    };

    let delta = match chars.next()
    {
        Some('^') => 1,
        Some('_') => -1,
        Some('\'') => 12,
        Some(',') => -12,
        Some('=') => 0,
        None => 0,
        Some(c) => panic!("Unexpected character: '{}'", c)
    };

    midi += delta;

    while let Some(octave_shift) = chars.next()
    {
        match octave_shift
        {
            '\'' => midi += 12,
            ',' => midi -= 12,
            c => panic!("Unexpected character: '{}'", c)
        }
    }

    midi
}

pub fn note_to_abc(note: &str) -> &'static str
{
    midi_to_abc(note_to_midi(note))
}

pub fn midi_to_flat(note: i8) -> &'static str
{
    MIDSCRIPT_FLATS[note as usize]
}

pub fn midi_to_sharp(note: i8) -> &'static str
{
    MIDSCRIPT_SHARPS[note as usize]
}

pub fn midi_to_abc(note: i8) -> &'static str
{
    ABC_NOTES[note as usize]
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_note_to_midi()
    {
        fn test(note: &str, midi: i8) { assert_eq!(note_to_midi(note), midi); }

        test("C,,,,,", 0);
        test("C", 60);
        test("C^", 61);
        test("C_", 59);
        test("a", 69);
        test("a=", 69);
        test("c", 72);
        test("c',", 72);
        test("c^',", 73);
        test("c_,'", 71);
        test("g''''", 127);
    }

    #[test]
    fn test_note_to_abc()
    {
        fn test(note: &str, abc: &str) { assert_eq!(note_to_abc(note), abc); }

        test("A", "A,");
        test("B", "B,");
        test("C", "C");
        test("D", "D");
        test("E", "E");
        test("a", "A");
        test("b", "B");
        test("c", "c");
        test("d", "d");
        test("e", "e");
        test("G^", "^G");
        test("G_", "^F");
        test("C,,", "C,,");
        test("D,,", "D,,");
        test("F^,,", "^F,,");
    }

    #[test]
    fn round_trip_sharp_conversions()
    {
        for i in 0..128
        {
            let i = i as i8;
            assert_eq!(note_to_midi(midi_to_sharp(i)), i);
        }
    }

    #[test]
    fn round_trip_flat_conversions()
    {
        for i in 0..128
        {
            let i = i as i8;
            assert_eq!(note_to_midi(midi_to_flat(i)), i);
        }
    }
}

