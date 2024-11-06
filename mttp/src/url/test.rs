use crate::url::parse_query_params_and_urldecode;
use std::collections::HashMap;

#[test]
fn test_parse_query1() {
    let s = "https://example.com/over/there?name=ferret";
    let got = parse_query_params_and_urldecode(s);

    let expected = (
        "https://example.com/over/there",
        HashMap::from([("name".to_owned(), "ferret".to_owned())]),
    );

    assert_eq!(got, expected);
}

#[test]
fn test_parse_query2() {
    let s = "https://example.com/path/to/page?name=ferret&color=purple";
    let got = parse_query_params_and_urldecode(s);

    let expected = (
        "https://example.com/path/to/page",
        HashMap::from([
            ("name".to_owned(), "ferret".to_owned()),
            ("color".to_owned(), "purple".to_owned()),
        ]),
    );

    assert_eq!(got, expected);
}

#[test]
fn test_parse_query3() {
    let s = "https://example.com/path?field1=value1&field1=value2&field2=value3";
    let got = parse_query_params_and_urldecode(s);

    let expected = (
        "https://example.com/path",
        HashMap::from([
            ("field1".to_owned(), "value2".to_owned()),
            ("field2".to_owned(), "value3".to_owned()),
        ]),
    );

    assert_eq!(got, expected);
}

#[test]
fn test_parse_query4() {
    let s = "https://www.google.com/search?q=%C3%BC%C3%B6%C3%A4%2F%2F&client=firefox";
    let got = parse_query_params_and_urldecode(s);

    let expected = (
        "https://www.google.com/search",
        HashMap::from([
            ("q".to_owned(), "üöä//".to_owned()),
            ("client".to_owned(), "firefox".to_owned()),
        ]),
    );

    assert_eq!(got, expected);
}

#[test]
fn test_basic_url_with_params() {
    let url = "http://example.com/page?name=John&age=30";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("name".to_string(), "John".to_string());
    expected_params.insert("age".to_string(), "30".to_string());

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_without_params() {
    let url = "http://example.com/page";
    let (base, params) = parse_query_params_and_urldecode(url);
    let expected_params: HashMap<String, String> = HashMap::new();

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_with_encoded_params() {
    let url = "http://example.com/page?name=John%20Doe&city=New%20York";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("name".to_string(), "John Doe".to_string());
    expected_params.insert("city".to_string(), "New York".to_string());

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_with_empty_params() {
    let url = "http://example.com/page?";
    let (base, params) = parse_query_params_and_urldecode(url);
    let expected_params: HashMap<String, String> = HashMap::new();

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_with_special_characters() {
    let url = "http://example.com/page?query=space%20&query2=%40";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("query".to_string(), "space ".to_string());
    expected_params.insert("query2".to_string(), "@".to_string());

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_with_multiple_query_params() {
    let url = "http://example.com/?foo=1&bar=2&baz=3";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("foo".to_string(), "1".to_string());
    expected_params.insert("bar".to_string(), "2".to_string());
    expected_params.insert("baz".to_string(), "3".to_string());

    assert_eq!(base, "http://example.com/");
    assert_eq!(params, expected_params);
}

#[cfg(test)]
fn generate_large_query_params(num_params: usize) -> String {
    let base = "http://example.com/page?";
    let mut params = Vec::new();
    for i in 0..num_params {
        params.push(format!("key{}=value{}", i, i));
    }
    format!("{}{}", base, params.join("&"))
}

#[test]
fn test_large_input() {
    let large_url = generate_large_query_params(10_000);
    let (base, params) = parse_query_params_and_urldecode(&large_url);

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params.len(), 10_000);

    assert_eq!(params.get("key9999"), Some(&"value9999".to_string()));
    assert_eq!(params.get("key8888"), Some(&"value8888".to_string()));
}

#[test]
fn test_url_with_repeated_params() {
    let url = "http://example.com/page?name=John&name=Jane";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("name".to_string(), "Jane".to_string()); // Last occurrence should be kept

    assert_eq!(base, "http://example.com/page");
    assert_eq!(params, expected_params);
}

#[test]
fn test_url_with_no_base() {
    let url = "?name=John";
    let (base, params) = parse_query_params_and_urldecode(url);
    let mut expected_params = HashMap::new();
    expected_params.insert("name".to_string(), "John".to_string());

    assert_eq!(base, "");
    assert_eq!(params, expected_params);
}

#[test]
fn test_invalid_url() {
    let url = "invalid_url";
    let (base, params) = parse_query_params_and_urldecode(url);
    let expected_params: HashMap<String, String> = HashMap::new();

    assert_eq!(base, "invalid_url");
    assert_eq!(params, expected_params);
}
