extern crate melo;

#[macro_use]
extern crate pretty_assertions;


macro_rules! test_abc
{
    ($test_name: ident) => {
        #[test]
        fn $test_name()
        {
            let source = include_str!(concat!("abc_tests/", stringify!($test_name), ".melo"));
            let expected = include_str!(concat!("abc_tests/", stringify!($test_name), ".abc"));

            let result = melo::compile_to_abc(source, None).unwrap();
            assert_eq!(result, expected);
        }
    }
}


test_abc!(simple_drums);
test_abc!(variable_drum_bars);
test_abc!(variable_stave_drum_bars);
test_abc!(triple_time_expansion);
test_abc!(triplets);
test_abc!(quintuplets);
test_abc!(not_sextuplets);
test_abc!(voice_args);
test_abc!(octave_shift);
test_abc!(simple_melody);
test_abc!(complex_melody);
test_abc!(longer_melody);
test_abc!(fifths);

