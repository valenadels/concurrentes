use stack_exchange_processor::processor::process_data;
use stack_exchange_processor::structures::sites::Sites;

#[cfg(test)]
#[test]
fn process_sites_test() {
    let data_dir = "./data_test";
    let sites_info = process_data(4, data_dir).unwrap();
    let sites_info_struct: Sites = serde_json::from_str(&sites_info).unwrap();

    assert_eq!(sites_info_struct.padron, "108201");
    assert_eq!(sites_info_struct.sites.len(), 2);
    assert_eq!(sites_info_struct.tags.len(), 11);
    assert_eq!(sites_info_struct.totals.chatty_sites.len(), 2);
    assert_eq!(
        sites_info_struct.totals.chatty_sites.first().unwrap(),
        "academia.stackexchange.com"
    );
    assert_eq!(
        sites_info_struct.totals.chatty_sites.get(1).unwrap(),
        "askubuntu.com"
    );
    assert_eq!(
        sites_info_struct.totals.chatty_tags,
        vec![
            "application",
            "computer-science",
            "graduate-admissions",
            "recommendation-letter",
            "usb",
            "usb-drive",
            "mathematics",
            "phd",
            "professors",
            "kde"
        ]
    );
}
