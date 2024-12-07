//! Binary for `null_webhook`

// teach me
#![deny(clippy::pedantic)]
// // no unsafe
// #![forbid(unsafe_code)]
// sane unsafe
#![forbid(unsafe_op_in_unsafe_fn)]
// no unwrap
#![deny(clippy::unwrap_used)]
// no panic
#![deny(clippy::panic)]
// docs!
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use clap::Parser as _;

/// Command-line arguments for the server
#[derive(clap::Parser)]
#[clap(version)]
struct Args {
    /// Bind address for the server
    #[clap(env)]
    listen_address: std::net::SocketAddr,
}

fn main() -> eyre::Result<()> {
    let app_context = null_webhook::AppContext::new();

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        eprintln!("user requested shutdown...");
        shutdown_tx
            .send(null_webhook::Shutdown)
            .expect("termination channel send failed");
    })?;

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(null_webhook::Ready) = ready_rx.recv() {
            let notify_result = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);
            if let Err(err) = notify_result {
                eprintln!("error sending sd_notify Ready: {err}");
            }
        }
    });

    let Args { listen_address } = Args::parse();
    let args = null_webhook::Args::listen(listen_address);
    app_context
        .server_builder(&args)
        .set_ready_sender(ready_tx)
        .set_shutdown_receiver(shutdown_rx)
        .serve()?;
    Ok(())
}
