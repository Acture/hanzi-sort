#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JyutpingRecord {
    pub jyutping: Vec<String>,
    pub character: char,
}

impl JyutpingRecord {
    pub fn primary_jyutping(&self) -> Option<&str> {
        self.jyutping.first().map(String::as_str)
    }
}
