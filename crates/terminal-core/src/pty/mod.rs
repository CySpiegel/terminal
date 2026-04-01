//! PTY abstraction layer.
//!
//! Wraps `portable-pty` to provide a uniform interface across Unix and
//! Windows (ConPTY).

pub mod platform;

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};

/// A spawned PTY with read/write handles.
pub struct Pty {
    pair: portable_pty::PtyPair,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

/// Read half of the PTY.
pub struct PtyReader {
    reader: Box<dyn Read + Send>,
}

/// Write half of the PTY.
pub struct PtyWriter {
    writer: Box<dyn Write + Send>,
}

impl Pty {
    /// Spawn a new PTY with the default shell.
    pub fn spawn(cols: u16, rows: u16) -> Result<(PtyReader, PtyWriter, Self), PtyError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::Open(e.to_string()))?;

        let shell = platform::default_shell();
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(dirs_next::home_dir().unwrap_or_else(|| ".".into()));

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::Spawn(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::Clone(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::Clone(e.to_string()))?;

        Ok((
            PtyReader { reader },
            PtyWriter { writer },
            Self { pair, child },
        ))
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        self.pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::Resize(e.to_string()))
    }

    /// Wait for the child process to exit.
    pub fn wait(&mut self) -> Result<portable_pty::ExitStatus, PtyError> {
        self.child
            .wait()
            .map_err(|e| PtyError::Wait(e.to_string()))
    }
}

impl PtyReader {
    /// Read bytes from the PTY. Blocks until data is available.
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl PtyWriter {
    /// Write bytes to the PTY.
    pub fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(buf)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("failed to open PTY: {0}")]
    Open(String),
    #[error("failed to spawn shell: {0}")]
    Spawn(String),
    #[error("failed to clone PTY handle: {0}")]
    Clone(String),
    #[error("failed to resize PTY: {0}")]
    Resize(String),
    #[error("failed to wait for child: {0}")]
    Wait(String),
}
