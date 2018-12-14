use melo::parse;

use failure::Error;

fn main() -> Result<(), Error> {
    let target = std::env::args().nth(1).unwrap();
    let source = std::fs::read_to_string(&target)?;
    eprintln!("{}", target);
    let parse_tree = parse::parse(&source, None).unwrap();
    eprintln!("{:#?}", parse_tree);

    Ok(())
}
