//! HTTP server that does nothing, responding HTTP 200 to all requests

pub use server::Builder as ServerBuilder;
pub use server::Error as ServerError;

/// Command-line arguments for the server
#[must_use]
pub struct Args {
    /// Bind address for the server
    listen_address: std::net::SocketAddr,
    /// Log all accesses to stdout
    log_accesses: bool,
}
impl Args {
    /// Configure listenining with basic authentication
    pub fn listen(listen_address: std::net::SocketAddr) -> Self {
        Self {
            listen_address,
            log_accesses: false,
        }
    }
    /// Enables printing a message on all accesses
    pub fn log_accesses(mut self) -> Self {
        self.log_accesses = true;
        self
    }
}

/// Signal to cleanly terminate after finishing the current request (if any)
pub struct Shutdown;

/// Signal that the server is ready to receive requests
pub struct Ready;

mod server {
    use crate::{Args, Ready, Shutdown};
    use std::{net::SocketAddr, time::Duration};
    use tiny_http::Response;

    /// Configuration for an HTTP server
    #[must_use]
    pub struct Builder<'a> {
        args: &'a Args,
        ready_tx: Option<std::sync::mpsc::Sender<Ready>>,
        shutdown_rx: Option<std::sync::mpsc::Receiver<Shutdown>>,
    }

    impl Args {
        /// Returns an HTTP server builder
        pub fn as_server_builder(&self) -> Builder<'_> {
            Builder {
                args: self,
                ready_tx: None,
                shutdown_rx: None,
            }
        }
    }
    impl Builder<'_> {
        /// Sets the sender to be notified when the server is [`Ready`]
        pub fn set_ready_sender(mut self, ready_tx: std::sync::mpsc::Sender<Ready>) -> Self {
            self.ready_tx = Some(ready_tx);
            self
        }

        /// Sets the receiver for the server [`Shutdown`] signal
        pub fn set_shutdown_receiver(
            mut self,
            shutdown_rx: std::sync::mpsc::Receiver<Shutdown>,
        ) -> Self {
            self.shutdown_rx = Some(shutdown_rx);
            self
        }

        /// Spawn a blocking HTTP server on the address specified by args
        ///
        /// # Errors
        ///
        /// Returns an error for any of the following:
        /// - binding the server fails
        /// - fail-fast metrics creation fails
        /// - shutdown receive fails (only if a `Receiver` was provided)
        /// - loading the auth key file fails
        ///
        pub fn serve(self) -> Result<(), Error> {
            let Self {
                args:
                    args @ Args {
                        listen_address,
                        log_accesses: _,
                    },
                mut ready_tx,
                mut shutdown_rx,
            } = self;

            let server = tiny_http::Server::http(listen_address).map_err(|io_error| Error {
                io_error,
                kind: ErrorKind::ServerBind {
                    listen_address: listen_address.to_owned(),
                },
            })?;

            println!("Listening at http://{listen_address:?} (and will reply to all HTTP requests with empty body, OK 200)");

            if let Some(ready_tx) = ready_tx.take() {
                // ignore "ready" receive errors
                let _ = ready_tx.send(Ready);
            }

            while Self::check_shutdown(shutdown_rx.as_mut()).is_none() {
                let _ = Self::serve_next_peer(&server, args);
            }
            Ok(())
        }
        fn check_shutdown(
            shutdown_rx: Option<&mut std::sync::mpsc::Receiver<Shutdown>>,
        ) -> Option<Shutdown> {
            shutdown_rx
                .map(|rx| rx.try_recv())
                .transpose()
                .unwrap_or_else(|err| {
                    use std::sync::mpsc::TryRecvError as E;
                    match err {
                        E::Disconnected => {
                            eprintln!("termination channel receive failure");
                            Some(Shutdown)
                            // TODO log an error to the console
                        }
                        E::Empty => {
                            // no shutdown signaled, yet
                            None
                        }
                    }
                })
        }
        fn serve_next_peer(server: &tiny_http::Server, args: &Args) -> Result<(), Error> {
            const RECV_TIMEOUT: Duration = Duration::from_millis(100);
            const RECV_SLEEP: Duration = Duration::from_millis(10);
            const HTTP_STATUS_OK: u32 = 200;

            if let Some(request) = server
                .recv_timeout(RECV_TIMEOUT)
                .map_err(Box::new)
                .map_err(|io_error| Error {
                    io_error,
                    kind: ErrorKind::PeerRecv,
                })?
            {
                if args.log_accesses {
                    println!("{request:?}");
                }
                request
                    .respond(Response::empty(HTTP_STATUS_OK))
                    .map_err(Box::new)
                    .map_err(|io_error| Error {
                        io_error,
                        kind: ErrorKind::PeerSend,
                    })
            } else {
                std::thread::sleep(RECV_SLEEP);
                Ok(())
            }
        }
    }

    /// Error establishing the server or communication with peers
    #[derive(Debug)]
    pub struct Error {
        kind: ErrorKind,
        io_error: Box<dyn std::error::Error + Send + Sync>,
    }
    /// For creating the server (report error to caller)
    #[derive(Debug)]
    enum ErrorKind {
        ServerBind { listen_address: SocketAddr },
        PeerRecv,
        PeerSend,
    }
    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&*self.io_error)
            // match &self.kind {
            //     ErrorKind::ServerBind { io_error, .. } => Some(&**io_error),
            // }
        }
    }
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Self { io_error: _, kind } = self;
            match kind {
                ErrorKind::ServerBind { listen_address } => {
                    write!(f, "failed to bind HTTP server to {listen_address}")
                }
                ErrorKind::PeerRecv => {
                    write!(f, "failed to receive from client")
                }
                ErrorKind::PeerSend => {
                    write!(f, "failed to send to client")
                }
            }
        }
    }
}
