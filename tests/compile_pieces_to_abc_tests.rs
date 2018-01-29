extern crate midscript;


macro_rules! test_piece
{
    ($test_name: ident) => {
        #[test]
        fn $test_name()
        {
            let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/pieces/", stringify!($test_name), ".midscript"));
            let filename = concat!(env!("CARGO_MANIFEST_DIR"), "/pieces/", stringify!($test_name), ".midscript");

            match midscript::compile_to_abc(source, Some(filename))
            {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    panic!("Compilation failed!")
                }
            }
        }
    }
}


test_piece!(minimal_melody);
test_piece!(minimal_drums);
test_piece!(minimal_chords);
test_piece!(multiple_instruments);
test_piece!(time_and_tempo);
