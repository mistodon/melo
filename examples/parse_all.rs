use melo::parse;

use failure::Error;

fn main() -> Result<(), Error> {
    let mut root = std::env::current_dir()?;
    root.push("pieces");

    for entry in std::fs::read_dir(&root)? {
        let path = entry?.path();
        let source = std::fs::read_to_string(&path)?;

        eprintln!("{}", path.display());
        let parse_tree = parse::parse(&source, None).unwrap();
        eprintln!("{:?}", parse_tree);
    }

    Ok(())
}
