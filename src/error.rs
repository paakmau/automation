use std::result;

// TODO: Error handling
pub type Error = String;

pub type Result<T> = result::Result<T, Error>;
