extern crate midscript;


macro_rules! test_abc
{
    ($test_name: tt) => {
        test_compilation(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/", $test_name, ".midscript")),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/", $test_name, ".abc")))
    }
}


fn test_compilation(before: &str, after: &str)
{
    let result = midscript::compile_to_abc(before);
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
