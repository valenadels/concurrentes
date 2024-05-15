/// Enum representing possible errors that can occur while running the program.
#[derive(Debug)]
pub enum Error {
    ///Parsing error. For example, when reading program arguments o parsing json files.
    ParsingError(String),
    ///Error reading files or directories.
    ReadingError(String),
    ///Error related to rayon's thread pool.
    ThreadPoolError(String),
}
