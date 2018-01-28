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


#[derive(Debug)]
pub struct SourceLoc
{
    pub line: usize,
    pub col: usize,
    pub info: SourceMap,
}

impl SourceLoc
{
    pub fn line_col(&self) -> (usize, usize)
    {
        (self.line, self.col)
    }

    pub fn filename_or(&self, alternative: &'static str) -> &str
    {
        self.info
            .filename
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or(alternative)
    }

    pub fn cause_line(&self) -> &str
    {
        &self.info.lines[self.line - 1]
    }

    pub fn under_line(&self) -> String
    {
        format!("{: >col$}", "^", col = self.col)
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
