use serde::{Deserialize, Serialize};

/// This structure is used to map the data from the JSONL files
/// Those files contain some StackExchange's posts
#[derive(Deserialize, Serialize)]
pub struct SiteRawData {
    /// Text contains the title and the body of the post
    pub texts: [String; 2],
    /// Tags contains the tags of the post
    pub tags: Vec<String>,
}
