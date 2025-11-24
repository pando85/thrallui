use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::io::{Read, Write};

pub struct ProcessManager {
    pty_system: NativePtySystem,
}

impl std::fmt::Debug for ProcessManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessManager")
            .field("pty_system", &"<NativePtySystem>")
            .finish()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            pty_system: NativePtySystem::default(),
        }
    }

    /// Spawn Claude CLI process with PTY
    pub fn spawn_claude(&self, directory: &str, claude_path: &str) -> Result<PtyPair> {
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = self
            .pty_system
            .openpty(pty_size)
            .context("Failed to create PTY")?;

        let mut cmd = CommandBuilder::new(claude_path);
        cmd.cwd(directory);

        let _child = pty_pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn Claude process")?;

        Ok(pty_pair)
    }

    /// Read available output from PTY (non-blocking)
    pub fn read_output(reader: &mut Box<dyn Read + Send>) -> Result<String> {
        let mut buffer = [0u8; 4096];
        let mut output = String::new();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    output.push_str(&chunk);
                    if n < buffer.len() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
            }
        }

        Ok(output)
    }

    /// Write input to PTY
    pub fn write_input(writer: &mut Box<dyn Write + Send>, input: &str) -> Result<()> {
        writer.write_all(input.as_bytes())?;
        writer.flush()?;
        Ok(())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
