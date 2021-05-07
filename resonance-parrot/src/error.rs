use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;


#[derive(Debug, Clone)]
pub struct ResonanceParrotError {
    pub message: String,
}

impl ResonanceParrotError {
    pub fn new(err_str: &str)  -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::<ResonanceParrotError>::new( ResonanceParrotError{ message:err_str.to_string()})
    }
}

impl fmt::Display for ResonanceParrotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ResonanceParrotError {
    //fn description(&self) -> &str {
    //    &self.message
    //}
}

unsafe impl std::marker::Send for ResonanceParrotError {
 
}

#[derive(Debug, Clone)]
pub struct ResonanceParrotWarning {
    pub message: String,
}

#[allow(dead_code)]
impl ResonanceParrotWarning {
    pub fn new(err_str: &str)  -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::<ResonanceParrotWarning>::new( ResonanceParrotWarning{ message:err_str.to_string()})
    }
}

#[allow(dead_code)]
impl fmt::Display for ResonanceParrotWarning {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

#[allow(dead_code)]
impl std::error::Error for ResonanceParrotWarning {
    fn description(&self) -> &str {
        &self.message
    }
}