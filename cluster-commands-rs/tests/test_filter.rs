use serde_json::json;
use evtx_clustering::filter::{Filter, Matches, FilterRule};

#[test]
fn test_filter() {
    let filter = FilterRule::from_jmes("test == 'blah'").unwrap();
    let result = filter.matches(json!({"test": "blah"})).unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_filter_2() {
    let filter = FilterRule::from_jmes("test == 'BLAH'").unwrap();
    let result = filter.matches(json!({"test": "blah"})).unwrap();
    assert_eq!(result, false);
}


#[test]
fn test_or_filter() {
    let filter1 = FilterRule::from_jmes("test == 'BLAH'").unwrap();
    let filter2 = FilterRule::from_jmes("test == 'blah'").unwrap();

    let or_filter = Filter::OrFilter(vec![filter1, filter2]);

    let result = or_filter.matches(json!({"test": "blah"})).unwrap();
    assert_eq!(result, true);
}
