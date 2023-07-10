#[derive(Debug)]
pub struct Config {
    pub ident_space: usize,
    pub max_line_length: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // this being strictly > to 1 is assumed.
            ident_space: 2,
            max_line_length: 100,
        }
    }
}
