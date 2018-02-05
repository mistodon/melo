pub const MELO_SHARPS: [&str; 128] = [
    "C,,,,,", "C#,,,,,", "D,,,,,", "D#,,,,,", "E,,,,,", "F,,,,,", "F#,,,,,", "G,,,,,",
    "G#,,,,,", "A,,,,", "A#,,,,", "B,,,,", "C,,,,", "C#,,,,", "D,,,,", "D#,,,,", "E,,,,",
    "F,,,,", "F#,,,,", "G,,,,", "G#,,,,", "A,,,", "A#,,,", "B,,,", "C,,,", "C#,,,", "D,,,",
    "D#,,,", "E,,,", "F,,,", "F#,,,", "G,,,", "G#,,,", "A,,", "A#,,", "B,,", "C,,", "C#,,",
    "D,,", "D#,,", "E,,", "F,,", "F#,,", "G,,", "G#,,", "A,", "A#,", "B,", "C,", "C#,", "D,",
    "D#,", "E,", "F,", "F#,", "G,", "G#,", "A", "A#", "B", "C", "C#", "D", "D#", "E", "F",
    "F#", "G", "G#", "a", "a#", "b", "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a'",
    "a#'", "b'", "c'", "c#'", "d'", "d#'", "e'", "f'", "f#'", "g'", "g#'", "a''", "a#''",
    "b''", "c''", "c#''", "d''", "d#''", "e''", "f''", "f#''", "g''", "g#''", "a'''", "a#'''",
    "b'''", "c'''", "c#'''", "d'''", "d#'''", "e'''", "f'''", "f#'''", "g'''", "g#'''",
    "a''''", "a#''''", "b''''", "c''''", "c#''''", "d''''", "d#''''", "e''''", "f''''",
    "f#''''", "g''''",
];

pub const MIN_SHARP: &str = MELO_SHARPS[0];
pub const MAX_SHARP: &str = MELO_SHARPS[127];


pub const MELO_FLATS: [&str; 128] = [
    "C,,,,,", "D_,,,,,", "D,,,,,", "E_,,,,,", "E,,,,,", "F,,,,,", "G_,,,,,", "G,,,,,",
    "A_,,,,", "A,,,,", "B_,,,,", "B,,,,", "C,,,,", "D_,,,,", "D,,,,", "E_,,,,", "E,,,,",
    "F,,,,", "G_,,,,", "G,,,,", "A_,,,", "A,,,", "B_,,,", "B,,,", "C,,,", "D_,,,", "D,,,",
    "E_,,,", "E,,,", "F,,,", "G_,,,", "G,,,", "A_,,", "A,,", "B_,,", "B,,", "C,,", "D_,,",
    "D,,", "E_,,", "E,,", "F,,", "G_,,", "G,,", "A_,", "A,", "B_,", "B,", "C,", "D_,", "D,",
    "E_,", "E,", "F,", "G_,", "G,", "A_", "A", "B_", "B", "C", "D_", "D", "E_", "E", "F",
    "G_", "G", "a_", "a", "b_", "b", "c", "d_", "d", "e_", "e", "f", "g_", "g", "a_'", "a'",
    "b_'", "b'", "c'", "d_'", "d'", "e_'", "e'", "f'", "g_'", "g'", "a_''", "a''", "b_''",
    "b''", "c''", "d_''", "d''", "e_''", "e''", "f''", "g_''", "g''", "a_'''", "a'''",
    "b_'''", "b'''", "c'''", "d_'''", "d'''", "e_'''", "e'''", "f'''", "g_'''", "g'''",
    "a_''''", "a''''", "b_''''", "b''''", "c''''", "d_''''", "d''''", "e_''''", "e''''",
    "f''''", "g_''''", "g''''",
];

pub const MIN_FLAT: &str = MELO_FLATS[0];
pub const MAX_FLAT: &str = MELO_FLATS[127];


pub const ABC_NOTES: [&str; 128] = [
    "=C,,,,,", "^C,,,,,", "=D,,,,,", "^D,,,,,", "=E,,,,,", "=F,,,,,", "^F,,,,,", "=G,,,,,",
    "^G,,,,,", "=A,,,,,", "^A,,,,,", "=B,,,,,", "=C,,,,", "^C,,,,", "=D,,,,", "^D,,,,",
    "=E,,,,", "=F,,,,", "^F,,,,", "=G,,,,", "^G,,,,", "=A,,,,", "^A,,,,", "=B,,,,", "=C,,,",
    "^C,,,", "=D,,,", "^D,,,", "=E,,,", "=F,,,", "^F,,,", "=G,,,", "^G,,,", "=A,,,", "^A,,,",
    "=B,,,", "=C,,", "^C,,", "=D,,", "^D,,", "=E,,", "=F,,", "^F,,", "=G,,", "^G,,", "=A,,",
    "^A,,", "=B,,", "=C,", "^C,", "=D,", "^D,", "=E,", "=F,", "^F,", "=G,", "^G,", "=A,",
    "^A,", "=B,", "=C", "^C", "=D", "^D", "=E", "=F", "^F", "=G", "^G", "=A", "^A", "=B",
    "=c", "^c", "=d", "^d", "=e", "=f", "^f", "=g", "^g", "=a", "^a", "=b", "=c'", "^c'",
    "=d'", "^d'", "=e'", "=f'", "^f'", "=g'", "^g'", "=a'", "^a'", "=b'", "=c''", "^c''",
    "=d''", "^d''", "=e''", "=f''", "^f''", "=g''", "^g''", "=a''", "^a''", "=b''", "=c'''",
    "^c'''", "=d'''", "^d'''", "=e'''", "=f'''", "^f'''", "=g'''", "^g'''", "=a'''", "^a'''",
    "=b'''", "=c''''", "^c''''", "=d''''", "^d''''", "=e''''", "=f''''", "^f''''", "=g''''",
];


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Midi(i8);


impl Midi
{
    pub fn midi(&self) -> i8
    {
        self.0
    }

    pub fn transposed(&self, semitones: i8) -> Option<Midi>
    {
        let midi = self.0.checked_add(semitones)?;
        if midi >= 0
        {
            Some(Midi(midi))
        }
        else
        {
            None
        }
    }

    pub fn from_raw(midi: i8) -> Option<Midi>
    {
        if midi >= 0
        {
            Some(Midi(midi))
        }
        else
        {
            None
        }
    }

    pub fn from_note(note: &str) -> Option<Midi>
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
            _ => return None,
        };

        for ch in chars
        {
            let delta = match ch
            {
                '#' => 1,
                '_' => -1,
                '\'' => 12,
                ',' => -12,
                '=' => 0,
                _ => return None,
            };

            midi += delta;
        }

        if midi >= 0 && midi < 128
        {
            Some(Midi(midi as i8))
        }
        else
        {
            None
        }
    }

    pub fn to_abc(&self) -> &'static str
    {
        ABC_NOTES[self.0 as usize]
    }

    pub fn to_sharp(&self) -> &'static str
    {
        MELO_SHARPS[self.0 as usize]
    }

    pub fn to_flat(&self) -> &'static str
    {
        MELO_FLATS[self.0 as usize]
    }
}


pub fn lcm(a: u32, b: u32) -> u32
{
    if a == 0 || b == 0
    {
        return 0
    }
    let mut ra = a;
    let mut rb = b;
    while rb != 0
    {
        let t = rb;
        rb = ra % rb;
        ra = t;
    }
    (a / ra) * b
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_note_to_midi()
    {
        fn test(note: &str, midi: i8)
        {
            assert_eq!(Midi::from_note(note), Midi::from_raw(midi));
        }

        test("C,,,,,", 0);
        test("C", 60);
        test("C#", 61);
        test("C##", 62);
        test("C_", 59);
        test("C__", 58);
        test("a", 69);
        test("a=", 69);
        test("c", 72);
        test("c',", 72);
        test("c#',", 73);
        test("c_,'", 71);
        test("g''''", 127);
        test("A", 57);
        test("A#", 58);
        test("A_", 56);
        test("G#,", 56);
    }

    #[test]
    fn test_note_to_abc()
    {
        fn test(note: &str, abc: &str)
        {
            assert_eq!(Midi::from_note(note).unwrap().to_abc(), abc);
        }

        test("A", "=A,");
        test("B", "=B,");
        test("C", "=C");
        test("D", "=D");
        test("E", "=E");
        test("a", "=A");
        test("b", "=B");
        test("c", "=c");
        test("d", "=d");
        test("e", "=e");
        test("G#", "^G");
        test("G_", "^F");
        test("C,,", "=C,,");
        test("D,,", "=D,,");
        test("F#,,", "^F,,");
    }

    #[test]
    fn round_trip_sharp_conversions()
    {
        for i in 0..128
        {
            let i = i as i8;
            assert_eq!(
                Midi::from_note(Midi::from_raw(i).unwrap().to_sharp())
                    .unwrap()
                    .midi(),
                i
            );
        }
    }

    #[test]
    fn round_trip_flat_conversions()
    {
        for i in 0..128
        {
            let i = i as i8;
            assert_eq!(
                Midi::from_note(Midi::from_raw(i).unwrap().to_flat())
                    .unwrap()
                    .midi(),
                i
            );
        }
    }

    #[test]
    fn test_lcm()
    {
        fn test(a: u32, b: u32, expected: u32)
        {
            assert_eq!(lcm(a, b), expected);
        }

        test(0, 0, 0);
        test(1, 0, 0);
        test(0, 1, 0);
        test(1, 1, 1);
        test(2, 1, 2);
        test(1, 2, 2);
        test(3, 6, 6);
        test(4, 3, 12);
        test(6, 4, 12);
        test(2, 64, 64);
        test(7, 11, 77);
    }
}
