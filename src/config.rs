#[derive(Debug)]
pub struct Config {
    pub ident_space: u32,
    pub max_line_length: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ident_space: 2,
            max_line_length: std::u32::MAX,
        }
    }
}
