use super::urlencoding;
use std::collections::HashMap;

pub fn parse_query_params_and_urldecode<'a>(url: &'a str) -> (&'a str, HashMap<String, String>) {
    let Some(qm_index) = url.char_indices().find_map(|(i, c)| match c {
        '?' => Some(i),
        _ => None,
    }) else {
        return (url, HashMap::new());
    };

    let (url, params) = url.split_at(qm_index);

    let params = params.strip_prefix('?').unwrap_or(params);

    let params = params
        .split('&')
        .map(|pair| {
            let idx = pair.char_indices().find_map(|(i, c)| match c {
                '=' => Some(i),
                _ => None,
            });

            if let Some(idx) = idx {
                let (left, right) = pair.split_at(idx);
                let right = right.strip_prefix('=').unwrap_or(right);
                (left, right)
            } else {
                (pair, "")
            }
        })
        .map(|(k, v)| (urlencoding::decode_string(k), urlencoding::decode_string(v)))
        .map(|kv| match kv {
            (Some(k), Some(v)) => Some((k, v)),
            _ => None,
        })
        .filter_map(|x| x)
        .collect::<HashMap<_, _>>();

    (url, params)
}

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
