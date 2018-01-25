pub const MIDSCRIPT_SHARPS: [&str; 128] = [
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

pub const MIN_SHARP: &str = MIDSCRIPT_SHARPS[0];
pub const MAX_SHARP: &str = MIDSCRIPT_SHARPS[127];

pub const MIDSCRIPT_FLATS: [&str; 128] = [
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

pub const MIN_FLAT: &str = MIDSCRIPT_FLATS[0];
pub const MAX_FLAT: &str = MIDSCRIPT_FLATS[127];

pub const ABC_NOTES: [&str; 128] = [
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


pub fn note_to_midi(note: &str) -> Option<i8>
{
    let mut chars = note.chars();
    let mut midi: i64 = match chars.next().unwrap_or_default()
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
        _ => return None
    };

    let delta = match chars.next()
    {
        Some('^') => 1,
        Some('_') => -1,
        Some('\'') => 12,
        Some(',') => -12,
        Some('=') => 0,
        None => 0,
        Some(_) => return None
    };

    midi += delta;

    while let Some(octave_shift) = chars.next()
    {
        match octave_shift
        {
            '\'' => midi += 12,
            ',' => midi -= 12,
            _ => return None
        }
    }

    if midi >= 0 && midi < 128{ Some(midi as i8) } else { None }
}

pub fn note_to_abc(note: &str) -> Option<&'static str>
{
    note_to_midi(note).and_then(midi_to_abc)
}

pub fn midi_to_flat(note: i8) -> Option<&'static str>
{
    MIDSCRIPT_FLATS.get(note as usize).cloned()
}

pub fn midi_to_sharp(note: i8) -> Option<&'static str>
{
    MIDSCRIPT_SHARPS.get(note as usize).cloned()
}

pub fn midi_to_abc(note: i8) -> Option<&'static str>
{
    ABC_NOTES.get(note as usize).cloned()
}


pub fn lcm(a: u64, b: u64) -> u64
{
    let mut lcm = ::std::cmp::max(a, b);
    while !(lcm % a == 0 && lcm % b == 0)
    {
        lcm += 1
    }
    lcm
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_note_to_midi()
    {
        fn test(note: &str, midi: i8) { assert_eq!(note_to_midi(note).unwrap(), midi); }

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
        fn test(note: &str, abc: &str) { assert_eq!(note_to_abc(note).unwrap(), abc); }

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
            assert_eq!(note_to_midi(midi_to_sharp(i).unwrap()).unwrap(), i);
        }
    }

    #[test]
    fn round_trip_flat_conversions()
    {
        for i in 0..128
        {
            let i = i as i8;
            assert_eq!(note_to_midi(midi_to_flat(i).unwrap()).unwrap(), i);
        }
    }
}

