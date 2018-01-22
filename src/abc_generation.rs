use parsing::ParseTree;


pub fn generate_abc(parse_tree: &ParseTree) -> Option<String>
{
    let mut buffer = String::new();
    buffer.push_str("Good start!");
    Some(buffer)
}

