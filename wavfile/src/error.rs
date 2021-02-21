use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct WavFileError {
    pub message: String,
}

impl WavFileError {
    pub fn new(err_str: &str)  -> Box<dyn std::error::Error> {
        Box::<WavFileError>::new( WavFileError{ message:err_str.to_string()})
    }
}

impl fmt::Display for WavFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WavFileError {
    fn description(&self) -> &str {
        &self.message
    }
}
