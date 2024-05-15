use serde::{Deserialize, Serialize};

/// This structure is used to map the totals of all the sites.
/// It contains the N_SITES chatty sites and the N_TAGS chatty tags.
#[derive(Debug, Deserialize, Serialize)]
pub struct Totals {
    /// The N_SITES chatty sites ordered by the number of words per question
    pub chatty_sites: Vec<String>,
    /// The N_TAGS chatty tags ordered by the number of words per question for each tag
    pub chatty_tags: Vec<String>,
}
