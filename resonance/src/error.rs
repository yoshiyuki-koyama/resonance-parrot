use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


#[derive(Debug, Clone)]
pub struct ResonanceError {
    pub message: String,
}

impl ResonanceError {
    pub fn new(err_str: &str)  -> Box<dyn std::error::Error> {
        Box::<ResonanceError>::new( ResonanceError{ message:err_str.to_string()})
    }
}

impl fmt::Display for ResonanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ResonanceError {
    //fn description(&self) -> &str {
    //    &self.message
    //}
}

unsafe impl std::marker::Send for ResonanceError {
 
}

#[derive(Debug, Clone)]
pub struct ResonanceWarning {
    pub message: String,
}

impl ResonanceWarning {
    pub fn new(err_str: &str)  -> Box<dyn std::error::Error> {
        Box::<ResonanceWarning>::new( ResonanceWarning{ message:err_str.to_string()})
    }
}

impl fmt::Display for ResonanceWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ResonanceWarning {
    fn description(&self) -> &str {
        &self.message
    }
}