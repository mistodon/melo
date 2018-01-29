use std::fmt::{Error, Formatter};
use std::sync::Arc;


pub type SourceMap = Arc<SourceInfo>;


#[derive(Debug)]
pub struct SourceInfo
{
    pub filename: Option<String>,
    pub source: String,
    pub lines: Vec<String>,
}

impl SourceInfo
{
    pub fn new(source: &str, filename: Option<&str>) -> SourceMap
    {
        let filename = filename.map(str::to_owned);
        let source = source.to_owned();
        let lines = source.lines().map(str::to_owned).collect();

        Arc::new(SourceInfo {
            filename,
            source,
            lines,
        })
    }
}


#[derive(Debug, Clone)]
pub struct SourceLoc
{
    pub line: usize,
    pub col: usize,
    pub info: SourceMap,
    pub width: usize,
}

impl SourceLoc
{
    pub fn filename(&self) -> &str
    {
        self.info
            .filename
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or("<stdin>")
    }

    pub fn cause_line(&self) -> &str
    {
        &self.info.lines[self.line - 1]
    }

    pub fn text(&self) -> &str
    {
        &self.info.lines[self.line - 1][(self.col - 1)..(self.col + self.width - 1)]
    }
}

impl PartialEq for SourceLoc
{
    fn eq(&self, other: &Self) -> bool
    {
        self.line == other.line && self.col == other.col
            && Arc::ptr_eq(&self.info, &other.info)
    }
}

impl Eq for SourceLoc {}


pub fn fmt_error(
    f: &mut Formatter,
    message: &str,
    filename: &str,
    context: &str,
    line: usize,
    col: usize,
    width: usize,
) -> Result<(), Error>
{
    use ansi_term::Color;

    let red = Color::Fixed(9).bold();
    let blue = Color::Fixed(12).bold();
    let white = Color::Fixed(15).bold();

    let line_prefix = format!("{} |    ", line);
    let underline = format!(
        "{: >indent$}{}",
        "",
        "^".repeat(width),
        indent = col + line_prefix.len() - 1
    );

    writeln!(
        f,
        "{}: {}\n   {}: {}:{}:{}\n\n{}{}\n{}",
        red.paint("error"),
        white.paint(message),
        blue.paint("in"),
        filename,
        line,
        col,
        blue.paint(line_prefix),
        context,
        red.paint(underline)
    )
}
