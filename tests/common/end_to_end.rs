use super::MiniReqResult;
use crate::{
    assert_response,
    common::bin_cmd::{BinCommand, BinOutput},
    get_random_port_listen_addr, HTTP_OK,
};

struct Responses {
    metrics: MiniReqResult,
    root: MiniReqResult,
    unknown: MiniReqResult,
}

#[test]
fn empty() -> eyre::Result<()> {
    let listen_address = get_random_port_listen_addr()?;

    // startup server
    let (output, responses) = BinCommand::new()
        .arg(&listen_address)
        .spawn_cleanup_with(|| {
            // request from `/metrics` endpoint
            let metrics = minreq::get(format!("http://{listen_address}/metrics")).send();

            // request root `/`
            let root = minreq::get(format!("http://{listen_address}/")).send();

            // request non-existent URL
            let unknown = minreq::get(format!("http://{listen_address}/unknown")).send();

            Responses {
                metrics,
                root,
                unknown,
            }
        })?;

    {
        let BinOutput {
            status,
            stdout,
            stderr,
        } = output;

        // no fatal errors
        assert_eq!(stderr, "user requested shutdown...\n");
        assert_eq!(stdout, format!("Listening at http://{listen_address} (and will reply to all HTTP requests with empty body, OK 200)\n"));
        assert!(
            status.success(),
            "verify sleep duration after SIGINT, killing too early?"
        );
    }

    {
        let Responses {
            metrics,
            root,
            unknown,
        } = responses;

        assert_response("root", &root?, HTTP_OK, |content| {
            assert_eq!(content, "", "root");
            content.is_empty()
        });

        assert_response("unknown", &unknown?, HTTP_OK, |content| {
            assert_eq!(content, "", "unknown");
            content.is_empty()
        });

        assert_response("metrics", &metrics?, HTTP_OK, |content| {
            assert_eq!(content, "", "metrics");
            content.is_empty()
        });
    }

    Ok(())
}
