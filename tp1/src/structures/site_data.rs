use crate::structures::tag_info::TagsInfo;
use serde::{Deserialize, Serialize};

/// This structure is used to map the data of a site.
/// It contains the number of questions, the number of words, the tags and the chatty tags of the site.
#[derive(Debug, Deserialize, Serialize)]
pub struct SiteData {
    /// The number of questions of the site
    pub questions: u32,
    /// The number of words of the site (title + body)
    pub words: usize,
    /// The tags of the site
    pub tags: TagsInfo,
    /// 10 tags with greater relation words/question for the site
    pub chatty_tags: Vec<String>,
}
