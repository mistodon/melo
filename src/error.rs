use std::fmt::{Result, Formatter, Write};
use std::sync::Arc;

use formatting::{self, StyleType, Style};


pub mod colors
{
    pub use self::inner::*;
    use ansi_term::Style;

    const DEFAULTSTYLE: Style = Style {
        foreground: None,
        background: None,
        is_bold: false,
        is_dimmed: false,
        is_italic: false,
        is_underline: false,
        is_blink: false,
        is_reverse: false,
        is_hidden: false,
        is_strikethrough: false,
    };

    #[cfg(feature = "color")]
    mod inner
    {
        use super::*;
        use ansi_term::Colour;

        pub const RED: Style = Style {
            foreground: Some(Colour::Fixed(9)),
            is_bold: true,
            ..DEFAULTSTYLE
        };

        pub const YELLOW: Style = Style {
            foreground: Some(Colour::Fixed(11)),
            is_bold: true,
            ..DEFAULTSTYLE
        };

        pub const BLUE: Style = Style {
            foreground: Some(Colour::Fixed(12)),
            is_bold: true,
            ..DEFAULTSTYLE
        };

        pub const CYAN: Style = Style {
            foreground: Some(Colour::Fixed(14)),
            is_bold: true,
            ..DEFAULTSTYLE
        };

        pub const WHITE: Style = Style {
            foreground: Some(Colour::Fixed(15)),
            is_bold: true,
            ..DEFAULTSTYLE
        };
    }

    #[cfg(not(feature = "color"))]
    mod inner
    {
        use super::*;

        pub const RED: Style = DEFAULTSTYLE;
        pub const YELLOW: Style = DEFAULTSTYLE;
        pub const BLUE: Style = DEFAULTSTYLE;
        pub const CYAN: Style = DEFAULTSTYLE;
        pub const WHITE: Style = DEFAULTSTYLE;
    }
}


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
    pub info: SourceMap,
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
) -> Result
{
    use self::colors::{BLUE, RED, WHITE};

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
        RED.paint("error"),
        WHITE.paint(message),
        BLUE.paint("in"),
        filename,
        line,
        col,
        BLUE.paint(line_prefix),
        context,
        RED.paint(underline)
    )
}


pub fn fmt_error_multi<W: Write>(
    f: &mut W,
    style_type: &StyleType,
    message: &str,
    filename: &str,
    context: &str,
    line: usize,
    col: usize,
    width: usize,
) -> Result
{
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
        formatting::paint("error", Style::Red, style_type)?,
        formatting::paint(message, Style::White, style_type)?,
        formatting::paint("in", Style::Blue, style_type)?,
        filename,
        line,
        col,
        formatting::paint(&line_prefix, Style::Blue, style_type)?,
        context,
        formatting::paint(&underline, Style::Red, style_type)?,
    )
}

pub fn fmt_simple_error(f: &mut Formatter, message: &str, filename: &str)
    -> Result
{
    use self::colors::{BLUE, RED, WHITE};

    writeln!(
        f,
        "{}: {}\n   {}: {}",
        RED.paint("error"),
        WHITE.paint(message),
        BLUE.paint("in"),
        filename,
    )
}
