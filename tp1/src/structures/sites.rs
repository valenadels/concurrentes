use crate::structures::site_info::SitesInfo;
use crate::structures::tag_info::TagsInfo;
use crate::structures::totals::Totals;
use serde::{Deserialize, Serialize};

/// This structure is used to map the data of all the sites.
/// This is the final structure that will be returned.
#[derive(Debug, Deserialize, Serialize)]
pub struct Sites {
    /// The student's PADRON
    pub padron: String,
    /// The data of all the sites
    pub sites: SitesInfo,
    /// The tags of all the sites
    pub tags: TagsInfo,
    /// The totals of all the sites
    pub totals: Totals,
}
