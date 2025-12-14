use std::sync::Arc;

use http::{Request, header::HOST};
use switchboard_http_router::{
    Router,
    path::{PathTree, PathTreeMatched},
    rule::{HeaderMatch, QueryMatch, RegexOrExact, RuleBucket, RuleMatch},
};

fn build_parts(
    host: &str,
    path: &str,
    method: http::Method,
    headers: &[(&str, &str)],
    query: Option<&str>,
) -> http::request::Parts {
    let uri = if let Some(q) = query {
        format!("{}?{}", path, q)
    } else {
        path.to_string()
    };
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(HOST, host);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let req = req.body(()).unwrap();
    let (parts, _) = req.into_parts();
    parts
}

#[test]
fn match_hostname_exact_and_path_exact() {
    let mut router: Router<&'static str> = Router::new();
    let mut tree = PathTree::new();
    tree.add_matchit_route("/foo", RuleBucket::new_single("A"))
        .unwrap();
    router.hostname_tree.set("example.com", tree);

    let parts = build_parts("example.com", "/foo", http::Method::GET, &[], None);
    let matched = router.match_request_parts(&parts).expect("should match");
    match matched {
        PathTreeMatched::Matchit { matched } => {
            assert_eq!(matched.data, "A")
        }
        _ => panic!("unexpected match variant"),
    }
}

#[test]
fn match_hostname_wildcard_and_prefix() {
    let mut router: Router<&'static str> = Router::new();
    let mut tree = PathTree::new();
    // matchit uses wildcard segment; use a catch-all for prefix
    // matchit uses named wildcard segment to capture the rest
    tree.add_matchit_route("/v2/{*path}", RuleBucket::new_single("P"))
        .unwrap();
    router.hostname_tree.set("*.example.com", tree);

    let parts = build_parts("api.example.com", "/v2/foo", http::Method::GET, &[], None);
    let matched = router
        .match_request_parts(&parts)
        .expect("should match prefix");
    match matched {
        PathTreeMatched::Matchit { matched } => assert_eq!(matched.data, "P"),
        _ => panic!("unexpected match variant"),
    }
}

#[test]
fn match_regex_path_when_no_exact_or_prefix() {
    let mut router: Router<&'static str> = Router::new();
    let mut tree = PathTree::new();
    let re = regex::Regex::new("^/users/[0-9]+$").unwrap();
    tree.add_regex_route(re, RuleBucket::new_single("R"));
    router.hostname_tree.set("example.com", tree);

    let parts = build_parts("example.com", "/users/123", http::Method::GET, &[], None);
    let matched = router
        .match_request_parts(&parts)
        .expect("should match regex");
    match matched {
        PathTreeMatched::Regex { data, .. } => assert_eq!(data.data, "R"),
        _ => panic!("unexpected variant"),
    }
}

#[test]
fn rule_bucket_matches_method_header_and_query() {
    let mut router: Router<&'static str> = Router::new();
    let mut tree = PathTree::new();

    // Build a bucket with composite match: method + header + query
    let mut bucket = switchboard_http_router::rule::RuleBucket::new();
    let rule = RuleMatch {
        method: Some(http::Method::POST),
        headers: vec![HeaderMatch {
            header_name: http::header::HeaderName::from_static("x-version"),
            header_value: RegexOrExact::Exact(http::HeaderValue::from_static("v2")),
        }],
        queries: vec![QueryMatch {
            query_name: Arc::from("a"),
            query_value: RegexOrExact::Exact(Arc::from("111")),
        }],
    };
    bucket.add_rule(rule, "M");
    tree.add_matchit_route("/foo", bucket).unwrap();
    router.hostname_tree.set("example.com", tree);

    let parts = build_parts(
        "example.com",
        "/foo",
        http::Method::POST,
        &[("x-version", "v2")],
        Some("a=111&b="),
    );
    let matched = router
        .match_request_parts(&parts)
        .expect("should match composite rule");
    match matched {
        PathTreeMatched::Matchit { matched } => {
            assert_eq!(matched.data, "M");
            // method matched
            assert!(matched.matched.method_matched);
            // header matched one
            assert_eq!(matched.matched.header_matches.len(), 1);
            // query matched one
            assert_eq!(matched.matched.query_matches.len(), 1);
        }
        _ => panic!("unexpected variant"),
    }
}

#[test]
fn no_rule_in_selected_host_returns_404_like_error() {
    let mut router: Router<&'static str> = Router::new();
    let mut tree = PathTree::new();
    // Only route for /ok
    tree.add_matchit_route("/ok", RuleBucket::new_single("OK"))
        .unwrap();
    router.hostname_tree.set("example.com", tree);

    let parts = build_parts("example.com", "/miss", http::Method::GET, &[], None);
    let err = router
        .match_request_parts(&parts)
        .expect_err("should not match");
    match err {
        switchboard_http_router::error::Error::NoMatchRoute => {}
        _ => panic!("unexpected error kind"),
    }
}

#[test]
fn host_not_found_error() {
    let router: Router<&'static str> = Router::new();
    let parts = build_parts("unknown.com", "/foo", http::Method::GET, &[], None);
    let err = router
        .match_request_parts(&parts)
        .expect_err("should be host not found");
    match err {
        switchboard_http_router::error::Error::HostNotFound => {}
        _ => panic!("unexpected error kind"),
    }
}
