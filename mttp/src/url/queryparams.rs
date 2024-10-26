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
        .filter(|x| x.0 != "")
        .collect::<HashMap<_, _>>();

    (url, params)
}
