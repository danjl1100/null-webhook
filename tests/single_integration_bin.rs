//! Single integration test binary
//!
//! NOTE: Since the crate is primarily a "binary crate", the integration tests (running the
//! executable) are more important than library unit tests.
//!
//! As a general rule, there should only be one integration test binary, since integration tests
//! are run sequentially by cargo.
//!
//! Add as many `#[test]`s as you want! (in submodules of this `single_integration_bin`)

#![allow(clippy::panic)] // Tests can panic

mod common {
    const LISTEN_ADDRESS: &str = "127.0.0.1:9582";

    type MiniReqResult = Result<minreq::Response, minreq::Error>;

    mod end_to_end;

    mod bin_cmd;
}
const HTTP_OK: i32 = 200;

fn assert_response(
    label: &'static str,
    response: &minreq::Response,
    code: i32,
    check_fn: impl FnOnce(&str) -> bool,
) {
    let Ok(content) = response.as_str() else {
        panic!("expected UTF-8 response string for {label}");
    };

    assert_eq!(response.status_code, code, "{label} code");

    assert!(check_fn(content), "{label} check");
}
