use crate::error::Error;
use crate::structures::site_data::SiteData;
use crate::structures::site_info::SitesInfo;
use crate::structures::site_raw_data::SiteRawData;
use crate::structures::sites::Sites;
use crate::structures::tag_data::TagData;
use crate::structures::tag_info::TagsInfo;
use crate::structures::totals::Totals;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// The student's PADRON
const PADRON: &str = "108201";
/// Number of tags to be shown in chatty tags
const N_TAGS: usize = 10;
/// Chatty sites to be shown
const N_SITES: usize = 10;

/// This function receives a SitesInfo structure containing the data of all the sites
/// and returns a Sites structure for the final response.
///
/// # Arguments
/// * `sites_info` - A SitesInfo structure containing the data of all the sites
///
/// # Returns
/// * A SitesInfo structure with the data of all the sites, the tags and the totals
/// * An Error if something goes wrong during the conversion
pub fn build_response_site_info(sites_info: SitesInfo) -> Result<Sites, Error> {
    let tags = map_tags(&sites_info);
    Ok(Sites {
        totals: map_totals(&sites_info, &tags)?,
        tags,
        sites: sites_info,
        padron: PADRON.to_string(),
    })
}

/// This function receives a reference to a SitesInfo structure and a reference to a
/// TagsInfo structure and returns a Totals structure for the final response.
/// The Totals structure contains the N_SITES chatty sites and the N_TAGS chatty tags among all the sites.
///
/// # Arguments
/// * `sites_info` - A reference to a SitesInfo structure
/// * `tags` - A reference to a TagsInfo structure
///
/// # Returns
/// * A Totals structure with the N_SITES chatty sites and the N_TAGS chatty tags
/// * An Error if something goes wrong during the conversion
fn map_totals(sites_info: &SitesInfo, tags: &TagsInfo) -> Result<Totals, Error> {
    let mut sites = sites_info.iter().collect::<Vec<_>>();
    sites.sort_by(|a, b| {
        compare_ratios(a.0, a.1.words, a.1.questions, b.0, b.1.words, b.1.questions)
    });

    let chatty_sites = sites
        .iter()
        .take(N_SITES)
        .map(|(site, _)| site.to_string())
        .collect::<Vec<_>>();

    let chatty_tags = calculate_chatty_tags(tags);

    Ok(Totals {
        chatty_sites,
        chatty_tags,
    })
}

/// This function receives a reference to a SitesInfo structure and returns a TagsInfo structure for the final response
/// containing the tags and the TagData for all the sites.
/// The TagsInfo structure contains the tags as keys and the TagData as values.
/// The TagData contains the number of questions and the number of words.
///
/// # Arguments
/// * `sites_info` - A reference to a SitesInfo structure
///
/// # Returns
/// * A TagsInfo structure with the tags and the TagData
fn map_tags(sites_info: &SitesInfo) -> TagsInfo {
    let mut tags = TagsInfo::new();
    for site in sites_info.values() {
        for (tag, tag_data) in site.tags.iter() {
            let entry_tag_data = tags.entry(tag.to_string()).or_insert(TagData {
                questions: 0,
                words: 0,
            });
            entry_tag_data.questions += tag_data.questions;
            entry_tag_data.words += tag_data.words;
        }
    }
    tags
}

/// Reads a JSONL file and returns a SitesInfo structure.
/// The SitesInfo structure contains the site name and the Site structure.
/// The Site structure contains the number of questions, the number of words, the tags and the chatty tags.
///
/// # Arguments
/// * `file_path` - A string that holds the file path.
pub fn read_jsonl(file_path: &str) -> Result<SitesInfo, Error> {
    let file = File::open(file_path).map_err(|e| Error::ReadingError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut questions: u32 = 0;
    let mut total_words = 0;
    let mut tags = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| Error::ReadingError(e.to_string()))?;
        if let Ok(json_data) = serde_json::from_str::<SiteRawData>(&line) {
            questions += 1;
            let line_words = count_words(&json_data);
            total_words += line_words;
            save_tags_for_site(&json_data.tags, &mut tags, line_words);
        }
    }

    let chatty_tags = calculate_chatty_tags(&tags);
    let site = SiteData {
        questions,
        words: total_words,
        tags,
        chatty_tags,
    };
    let mut site_info = SitesInfo::new();
    site_info.insert(extract_site_name(file_path)?, site);

    Ok(site_info)
}

/// Extracts the site name from a given file path.
///
/// # Arguments
///
/// * `file_path` - A string that holds the file path.
///
/// # Returns
///
/// * `Ok(String)` - The site name extracted from the file path.
/// * `Err(Error)` - An error that occurred during the extraction of the site name.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The file path does not contain a '/' character, which means that the file name cannot be extracted.
/// * The file name does not contain minimum 3 parts, which means that the site name cannot be extracted.
///
/// # Example
///
/// "./data/mathematica.stackexchange.com.jsonl" -> "mathematica.stackexchange.com"
///
fn extract_site_name(file_path: &str) -> Result<String, Error> {
    let file_name = file_path.split('/').last().ok_or(Error::ParsingError(
        "Error extracting file name".to_string(),
    ))?;
    let site_name_parts: Vec<&str> = file_name.split('.').collect();
    if site_name_parts.len() < 2 {
        return Err(Error::ParsingError(
            "Error extracting site name".to_string(),
        ));
    }
    Ok(site_name_parts[0..site_name_parts.len() - 1].join("."))
}

/// Collects the tags from a given site in a HashMap.
/// The HashMap contains the tags as keys and the TagData as values.
/// The TagData contains the number of questions and the number of words.
/// If a tag is already in the HashMap, the function increments the number of questions and the number of words.
/// If a tag is not in the HashMap, the function inserts the tag with the number of questions and the number of words.
///
/// # Arguments
/// * `site_tags` - A reference to a vector of strings that holds the tags of a site.
/// * `tags_map` - A mutable reference to a HashMap that holds the tags and the TagData.
/// * `words` - The number of words in the texts field of the site.
fn save_tags_for_site(
    site_tags: &Vec<String>,
    tags_map: &mut HashMap<String, TagData>,
    words: usize,
) {
    for tag in site_tags {
        let tag_data = tags_map.entry(tag.to_string()).or_insert(TagData {
            questions: 0,
            words: 0,
        });
        tag_data.questions += 1;
        tag_data.words += words;
    }
}

/// This function receives a reference to a TagsInfo structure and returns the N_TAGS tags with the highest ratio of words per question.
/// If the ratio is the same, the tags are compared lexicographically in ascending order.
///
/// # Arguments
/// * `tag_info` - A reference to a TagsInfo structure
///
/// # Returns
/// * An array with the N_TAGS tags with the highest ratio of words per question or an Error if the conversion fails
fn calculate_chatty_tags(tag_info: &TagsInfo) -> Vec<String> {
    let mut tags: Vec<(&String, &TagData)> = tag_info.iter().collect::<Vec<_>>();
    tags.sort_by(|a, b| {
        compare_ratios(a.0, a.1.words, a.1.questions, b.0, b.1.words, b.1.questions)
    });

    tags.iter()
        .take(N_TAGS)
        .map(|(tag_name, _)| tag_name.to_string())
        .collect::<Vec<_>>()
}

/// This function receives two keys and their respective number of words and questions and compares them
/// based on the ratio of words per question. If the ratio is the same, the keys are compared lexicographically in ascending order.
/// The function returns the ordering of the keys.
///
/// # Arguments
/// * `key_a` - A reference to a String that holds the first key
/// * `a_words` - The number of words for the first key
/// * `a_questions` - The number of questions for the first key
/// * `key_b` - A reference to a String that holds the second key
/// * `b_words` - The number of words for the second key
/// * `b_questions` - The number of questions for the second key
fn compare_ratios(
    key_a: &String,
    a_words: usize,
    a_questions: u32,
    key_b: &String,
    b_words: usize,
    b_questions: u32,
) -> std::cmp::Ordering {
    let a_ratio = a_words as f64 / a_questions as f64;
    let b_ratio = b_words as f64 / b_questions as f64;

    let ratio_ordering = b_ratio
        .partial_cmp(&a_ratio)
        .unwrap_or(std::cmp::Ordering::Equal);

    if ratio_ordering == std::cmp::Ordering::Equal {
        key_a.cmp(key_b)
    } else {
        ratio_ordering
    }
}

/// This function receives a reference to a SiteRawData structure and returns the number of words in the texts field
/// That is, the number of words in the title and the body of the post
///
/// # Arguments
///
/// * `json_data` - A reference to a SiteRawData structure
///
/// # Returns
///
/// * The number of words in the texts field
fn count_words(json_data: &SiteRawData) -> usize {
    json_data.texts[0].split_whitespace().count() + json_data.texts[1].split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_ratios_orders_by_ratio_then_lexicographically() {
        let a = ("tag1".to_string(), &(2, 1));
        let b = ("zag2".to_string(), &(2, 1));
        let c = ("tag3".to_string(), &(3, 1));

        assert_eq!(
            compare_ratios(&a.0, a.1 .0, a.1 .1, &b.0, b.1 .0, b.1 .1),
            std::cmp::Ordering::Less
        ); // 2/1 = 2/1 --> tag1 < zag2
        assert_eq!(
            compare_ratios(&a.0, a.1 .0, a.1 .1, &c.0, c.1 .0, c.1 .1),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn calculate_chatty_tags_returns_expected_tags() {
        let tag_info = (1..=10)
            .map(|i| {
                (
                    format!("tag{}", i),
                    TagData {
                        questions: 1,
                        words: i,
                    },
                )
            })
            .collect::<TagsInfo>();

        let result = calculate_chatty_tags(&tag_info);
        assert_eq!(
            result,
            ["tag10", "tag9", "tag8", "tag7", "tag6", "tag5", "tag4", "tag3", "tag2", "tag1"]
        );
    }

    #[test]
    fn calculate_chatty_tags_less_tan_10_tags() {
        let tag_info = (1..=3)
            .map(|i| {
                (
                    format!("tag{}", i),
                    TagData {
                        questions: 1,
                        words: i,
                    },
                )
            })
            .collect::<TagsInfo>();

        let result = calculate_chatty_tags(&tag_info);
        assert_eq!(result, ["tag3", "tag2", "tag1"]);
    }

    #[test]
    fn read_jsonl_returns_error_for_invalid_file() {
        let result = read_jsonl("./data/nonexistent.jsonl");
        assert!(result.is_err());
    }

    #[test]
    fn extract_site_name_extracts_name_correctly() {
        let result = extract_site_name("./data/test.stackexchange.com.jsonl").unwrap();
        assert_eq!(result, "test.stackexchange.com");
    }

    #[test]
    fn extract_site_name_returns_error_for_invalid_path() {
        let result = extract_site_name("invalidpath");
        assert!(result.is_err());
    }

    #[test]
    fn count_words_counts_words_correctly() {
        let json_data = SiteRawData {
            texts: <[String; 2]>::try_from(vec![
                "Hello world".to_string(),
                "This is a test".to_string(),
            ])
            .unwrap(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        let result = count_words(&json_data);
        assert_eq!(result, 6);
    }
}
