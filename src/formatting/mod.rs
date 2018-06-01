use std::fmt::{self, Write, Error};


pub trait MultiFormat<FormatStyleType>
{
    fn multi_fmt<W: Write>(&self, f: &mut W, style_type: &FormatStyleType) -> fmt::Result;
}

#[derive(Debug)]
pub enum StyleType<'a>
{
    Normal,
    AnsiTerm,
    Html { css_class_prefix: &'a str },
}

#[derive(Debug, Clone, Copy)]
pub enum Style
{
    Normal,
    Red,
    Blue,
    Yellow,
    Cyan,
    White,
}

#[derive(Debug)]
pub struct Styled<'a>
{
    content: &'a str,
    style: Style,
}

impl<'a> Styled<'a>
{
    pub fn new(content: &'a str, style: Style) -> Self
    {
        Styled { content, style }
    }
}

pub fn paint(text: &str, style: Style, style_type: &StyleType) -> Result<String, Error>
{
    let mut buffer = String::new();
    let styled = Styled::new(text, style);
    styled.multi_fmt(&mut buffer, style_type)?;
    Ok(buffer)
}


impl<'a> MultiFormat<StyleType<'a>> for Styled<'a>
{
    fn multi_fmt<W: Write>(&self, f: &mut W, style_type: &StyleType) -> fmt::Result
    {
        use self::Style::*;

        match *style_type
        {
            StyleType::Normal => write!(f, "{}", self.content),
            StyleType::AnsiTerm => {
                use ansi_term::{Color, Style};

                let style = match self.style
                {
                    Normal => Style::new(),
                    Red => Color::Fixed(9).bold(),
                    Yellow => Color::Fixed(11).bold(),
                    Blue => Color::Fixed(12).bold(),
                    Cyan => Color::Fixed(13).bold(),
                    White => Color::Fixed(15).bold(),
                };

                write!(f, "{}", style.paint(self.content))
            }
            StyleType::Html { css_class_prefix } => {
                match self.style
                {
                    Normal => write!(f, "{}", self.content),
                    _ => {
                        let colorname = match self.style
                        {
                            Red => "red",
                            Blue => "blue",
                            Yellow => "yellow",
                            Cyan => "cyan",
                            White => "white",
                            Normal => unreachable!()
                        };

                        write!(f, "<span class='{}{}'>{}</span>",
                            css_class_prefix,
                            colorname,
                            self.content)
                    }
                }
            }
        }
    }
}
