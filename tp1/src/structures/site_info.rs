use crate::structures::site_data::SiteData;
use std::collections::HashMap;

/// This type is used to map the data (words, questions, tags and chatty tags) of all the sites.
/// The key is the site name.
/// The value is the site data.
pub type SitesInfo = HashMap<String, SiteData>;
