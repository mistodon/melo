use std::fmt::{Error, Formatter};
use std::sync::Arc;


#[derive(Debug)]
pub struct SourceMap<'a>
{
    pub filename: String,
    pub source: &'a str,
    line_boundaries: Vec<usize>,
    end_line_col: (usize, usize),
}

impl<'a> SourceMap<'a>
{
    pub fn new(filename: String, source: &'a str) -> Self
    {
        let mut byte_pos = 0;
        let mut line_boundaries = Vec::new();
        let mut last_line_len = 0;

        for line in source.lines()
        {
            line_boundaries.push(byte_pos);
            byte_pos += line.len() + 1;
            last_line_len = line.len();
        }
        line_boundaries.push(byte_pos);

        let end_line = line_boundaries.len();
        let end_col = last_line_len;
        let end_line_col = (end_line, end_col);

        SourceMap { filename, source, line_boundaries, end_line_col }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize)
    {
        let mut line_start = 0;
        for (line, &byte_pos) in self.line_boundaries.iter().enumerate()
        {
            if byte_pos > offset
            {
                let col = offset - line_start + 1;
                let line = line + 1;
                return (line, col)
            }
            line_start = byte_pos;
        }
        self.end_line_col
    }
}


pub type SourceInfoPtr = Arc<SourceInfo>;


#[derive(Debug)]
pub struct SourceInfo
{
    pub filename: Option<String>,
    pub source: String,
    pub lines: Vec<String>,
}

impl SourceInfo
{
    pub fn new(source: &str, filename: Option<&str>) -> SourceInfoPtr
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

    pub fn filename(&self) -> &str
    {
        self.filename
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or("<stdin>")
    }
}


#[derive(Debug, Clone)]
pub struct SourceLoc
{
    pub line: usize,
    pub col: usize,
    pub info: SourceInfoPtr,
    pub width: usize,
}

impl SourceLoc
{
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

pub fn fmt_simple_error(f: &mut Formatter, message: &str, filename: &str)
    -> Result<(), Error>
{
    use ansi_term::Color;

    let red = Color::Fixed(9).bold();
    let blue = Color::Fixed(12).bold();
    let white = Color::Fixed(15).bold();

    writeln!(
        f,
        "{}: {}\n   {}: {}",
        red.paint("error"),
        white.paint(message),
        blue.paint("in"),
        filename,
    )
}
