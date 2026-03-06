#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinYinRecord {
    pub pinyin: Vec<String>,
    pub character: char,
}

impl PinYinRecord {
    pub fn primary_pinyin(&self) -> Option<&str> {
        self.pinyin.first().map(String::as_str)
    }
}
