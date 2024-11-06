use super::protocol::parse_request;
use crate::http::{HeaderMap, HttpRequest, Method};
use std::{collections::HashMap, io::Cursor};

#[test]
pub fn test_request1() {
    let x = b"GET /test1/test2?real=fake HTTP/1.1\r\nTest: Test\r\n\r\n";
    let got = parse_request(&mut x.as_slice()).unwrap();

    assert_eq!(
        got,
        HttpRequest {
            method: crate::http::Method::Get,
            raw_route: "/test1/test2?real=fake".to_owned(),
            headers: HeaderMap::from([("Test", "Test")]),
            body: None,
            route: "/test1/test2".to_owned(),
            params: HashMap::from([("real".to_owned(), "fake".to_owned())])
        }
    )
}

#[test]
pub fn test_request2() {
    let x =
        b"GET /test1/test2?real=fake HTTP/1.1\r\nTest: Test\r\nContent-Length: 27\r\n\r\nTHIS IS A TEST \n\0\0TEST TEST";
    let got = parse_request(&mut x.as_slice()).unwrap();

    assert_eq!(
        got,
        HttpRequest {
            method: crate::http::Method::Get,
            raw_route: "/test1/test2?real=fake".to_owned(),
            headers: HeaderMap::from([("Test", "Test"), ("Content-Length", "27")]),
            body: Some(b"THIS IS A TEST \n\0\0TEST TEST".to_vec()),
            route: "/test1/test2".to_owned(),
            params: HashMap::from([("real".to_owned(), "fake".to_owned())])
        }
    )
}

#[test]
pub fn test_request3() {
    let x =
        b"GET /test1/test2?real=fake HTTP/1.1\r\nTest: Test\r\nContent-Length: 20\r\n\r\nTHIS IS A TEST \n\0\0TEST TEST";
    let got = parse_request(&mut x.as_slice()).unwrap();

    assert_eq!(
        got,
        HttpRequest {
            method: crate::http::Method::Get,
            raw_route: "/test1/test2?real=fake".to_owned(),
            headers: HeaderMap::from([("Test", "Test"), ("Content-Length", "20")]),
            body: Some(b"THIS IS A TEST \n\0\0TE".to_vec()),
            route: "/test1/test2".to_owned(),
            params: HashMap::from([("real".to_owned(), "fake".to_owned())])
        }
    )
}

#[test]
pub fn test_request4() {
    let x = b"GET /\0\0";
    let got = parse_request(&mut x.as_slice());

    match got {
        Err(e) => match e {
            crate::Error::UnsupportedVersion => {}
            _ => {
                panic!("Wrong error");
            }
        },
        _ => {
            panic!("Wrong error");
        }
    }
}

#[test]
pub fn test_request5() {
    let x = b"GET / HTTP/1.2";
    let got = parse_request(&mut x.as_slice());

    assert!(got.is_err());
}

#[test]
fn test_parse_request_with_empty_headers() {
    let request_data = b"GET /some/route HTTP/1.1\r\n\r\n";
    let mut cursor = Cursor::new(request_data);

    let result = parse_request(&mut cursor);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.raw_route, "/some/route");
    assert!(request.headers.is_empty());
    assert!(request.body.is_none());
    assert_eq!(request.route, "/some/route");
    assert!(request.params.is_empty());
}

#[test]
fn test_parse_request_with_missing_method() {
    let request_data = b"/some/route HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let mut cursor = Cursor::new(request_data);

    let result = parse_request(&mut cursor);

    assert!(result.is_err()); // Expecting an error for missing method
}

#[test]
fn test_parse_invalid_http_request() {
    let request_data = b"INVALID REQUEST FORMAT";
    let mut cursor = Cursor::new(request_data);

    let result = parse_request(&mut cursor);

    assert!(result.is_err()); // Expecting an error for invalid format
}

#[test]
fn test_parse_request_with_body() {
    let request_data = b"POST /some/route HTTP/1.1\r\nHost: example.com\r\nContent-Length: 13\r\n\r\nHello, world!";
    let mut cursor = Cursor::new(request_data);

    let result = parse_request(&mut cursor);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.raw_route, "/some/route");
    assert_eq!(request.headers.get("Host").unwrap(), "example.com");
    assert_eq!(request.body.as_deref(), Some(b"Hello, world!".as_ref()));
    assert_eq!(request.route, "/some/route");
    assert!(request.params.is_empty());
}

#[test]
fn test_parse_valid_http_request() {
    let request_data = b"GET /some/route HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let mut cursor = Cursor::new(request_data);

    let result = parse_request(&mut cursor);

    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.method, Method::Get); // Adjust according to your Method enum
    assert_eq!(request.raw_route, "/some/route");
    assert_eq!(request.headers.get("Host").unwrap(), "example.com");
    assert!(request.body.is_none());
    assert_eq!(request.route, "/some/route"); // Assuming route parsing is just the raw route
    assert!(request.params.is_empty());
}
