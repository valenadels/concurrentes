use crate::structures::tag_data::TagData;
use std::collections::HashMap;

/// This type is used to map the data (questions and words) of all the tags.
/// The key is the tag name.
/// The value is the tag data.
pub type TagsInfo = HashMap<String, TagData>;
