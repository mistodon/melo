extern crate midscript;

#[macro_use]
extern crate pretty_assertions;


macro_rules! test_abc
{
    ($test_name: tt) => {
        test_compilation(
            include_str!(concat!("abc_tests/", $test_name, ".midscript")),
            include_str!(concat!("abc_tests/", $test_name, ".abc")))
    }
}


fn test_compilation(before: &str, after: &str)
{
    let result = midscript::compile_to_abc(before, None).unwrap();
    assert_eq!(result, after);
}


#[test]
fn test_simple_drums()
{
    test_abc!("simple_drums");
}

#[test]
fn test_variable_length_drum_bars()
{
    test_abc!("variable_drum_bars");
}

#[test]
fn test_variable_length_drum_staves()
{
    test_abc!("variable_stave_drum_bars");
}

#[test]
fn test_triple_time_expansion()
{
    test_abc!("triple_time_expansion");
}

#[test]
fn test_automatic_triplets()
{
    test_abc!("triplets");
}

#[test]
fn test_automatic_quintuplets()
{
    test_abc!("quintuplets");
}

#[test]
fn test_triplets_and_not_sextuplets()
{
    test_abc!("not_sextuplets");
}

#[test]
fn test_voice_args()
{
    test_abc!("voice_args")
}

#[test]
fn test_octave_shift()
{
    test_abc!("octave_shift")
}

#[test]
fn test_simple_melody()
{
    test_abc!("simple_melody")
}

#[test]
fn test_complex_melody()
{
    test_abc!("complex_melody")
}

#[test]
fn test_longer_melody()
{
    test_abc!("longer_melody")
}

#[test]
fn test_fifths()
{
    test_abc!("fifths")
}
