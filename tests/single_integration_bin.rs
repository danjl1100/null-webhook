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

use eyre::Context as _;

mod common {
    type MiniReqResult = Result<minreq::Response, minreq::Error>;

    mod end_to_end;
    mod end_to_end_log_accesses;

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

fn get_random_port_listen_addr() -> eyre::Result<String> {
    // Bind to a random port to occupy it
    let listener = std::net::TcpListener::bind("127.0.0.1:0").context("bind for random port")?;
    let addr = listener
        .local_addr()
        .context("get local address from bind")?;
    let port = addr.port();

    Ok(format!("127.0.0.1:{port}"))
    // let ip = Ipv4Addr::new(127, 0, 0, 1).into();
    // Ok(SocketAddr::new(ip, port))
}
