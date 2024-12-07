use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::{
    process::{Child, Command, ExitStatus, Output, Stdio},
    time::Duration,
};

#[derive(Default)]
pub struct BinCommand {
    args: Vec<String>,
}
impl BinCommand {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn arg(mut self, arg: &'static str) -> Self {
        self.args.push(arg.to_string());
        self
    }
    // TODO remove
    // pub fn arg_dynamic(mut self, arg: String) -> Self {
    //     self.args.push(arg);
    //     self
    // }
    fn build(self) -> Command {
        const BIN_EXE: &str = env!("CARGO_BIN_EXE_null-webhook");

        let mut command = Command::new(BIN_EXE);

        if !self.args.is_empty() {
            command.args(self.args);
        }

        command
    }
    pub fn spawn(self) -> std::io::Result<BinChild> {
        let subcommand = self
            .build()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // allow grace period for init
        std::thread::sleep(Duration::from_millis(100));

        Ok(BinChild { subcommand })
    }
    pub fn spawn_cleanup_with<T>(self, f: impl FnOnce() -> T) -> eyre::Result<(BinOutput, T)> {
        let mut child = self.spawn()?;
        let fn_output = f();

        child.interrupt_wait()?;
        let output = child.kill_await_output()?;

        Ok((output, fn_output))
    }
}

pub struct BinChild {
    subcommand: Child,
}
impl BinChild {
    pub fn interrupt_wait(&mut self) -> eyre::Result<()> {
        // SIGINT - request clean exit
        signal::kill(
            Pid::from_raw(self.subcommand.id().try_into()?),
            Signal::SIGINT,
        )?;

        // allow grace period for cleanup
        std::thread::sleep(Duration::from_millis(300));

        Ok(())
    }
    pub fn kill_await_output(mut self) -> eyre::Result<BinOutput> {
        self.subcommand.kill()?;

        let output = self.subcommand.wait_with_output()?;
        let output = BinOutput::new(output)?;

        Ok(output)
    }
    // TODO remove
    // pub fn is_finished(&mut self) -> std::io::Result<bool> {
    //     let wait_result = self.subcommand.try_wait()?;
    //     Ok(wait_result.is_some())
    // }
}

pub struct BinOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}
impl BinOutput {
    fn new(output: Output) -> Result<Self, std::string::FromUtf8Error> {
        let Output {
            status,
            stdout,
            stderr,
        } = output;
        let stdout = String::from_utf8(stdout)?;
        let stderr = String::from_utf8(stderr)?;
        Ok(Self {
            status,
            stdout,
            stderr,
        })
    }
}
