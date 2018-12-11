#![allow(deprecated)]

macro_rules! test_piece {
    ($test_name: ident) => {
        #[test]
        fn $test_name() {
            let source = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/pieces/",
                stringify!($test_name),
                ".melo"
            ));
            let filename = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/pieces/",
                stringify!($test_name),
                ".melo"
            );

            match melo::compile_to_abc(source, Some(filename)) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    panic!("Compilation failed!")
                }
            }
        }
    };
}

test_piece!(minimal_melody);
test_piece!(minimal_drums);
test_piece!(minimal_chords);
test_piece!(multiple_instruments);
test_piece!(time_and_tempo);
test_piece!(polyrhythms);
test_piece!(rondo_alla_turca);
test_piece!(too_many_staves);
test_piece!(repeats);
