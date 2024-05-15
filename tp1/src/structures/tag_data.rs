use serde::{Deserialize, Serialize};

/// This structure is used to map the data of a tag.
/// It contains the number of questions and the number of words of the tag.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagData {
    /// The number of questions of the tag
    pub questions: u32,
    /// The number of words of the tag
    pub words: usize,
}
