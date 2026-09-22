use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);
struct ManagedChild(Child);
impl ManagedChild {
    fn stop(&mut self) {
        #[cfg(unix)]
        // Every child starts a fresh group. SIGTERM lets the helper close Chromium.
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGTERM);
        }
        #[cfg(not(unix))]
        if let Some(stdin) = self.0.stdin.as_mut() {
            let _ = stdin.write_all(b"{\"cancel\":true}\n");
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.0.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Drop for ManagedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            self.stop();
        }
    }
}
pub fn run(command: &mut Command, input: Option<&Value>, timeout: Duration) -> Result<String> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = ManagedChild(
        command
            .spawn()
            .context("Could not start subprocess; check Node.js/npm installation and PATH")?,
    );
    if let Some(input) = input {
        writeln!(
            child.0.stdin.as_mut().unwrap(),
            "{}",
            serde_json::to_string(input)?
        )?;
    } else {
        child.0.stdin.take();
    }
    let mut pipe = child.0.stdout.take().unwrap();
    let capture = thread::spawn(move || {
        let mut s = String::new();
        let _ = pipe.read_to_string(&mut s);
        s
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if INTERRUPTED.load(Ordering::Relaxed) || started.elapsed() > timeout {
            child.stop();
            bail!(
                "Subprocess interrupted or timed out; temporary preview and browser are being closed"
            );
        }
        thread::sleep(Duration::from_millis(50));
    };
    let output = capture.join().unwrap_or_default();
    if !status.success() {
        bail!("Subprocess failed ({status}): {}", output.trim());
    }
    Ok(output)
}
pub fn helper(root: &Path, input: &Value) -> Result<Value> {
    let dir = root.join("tooling/renderer");
    if !dir.join("node_modules/playwright/package.json").is_file() {
        bail!(
            "Renderer dependencies missing. Run: presmith setup '{}' (requires Node.js 22+ and npm)",
            root.display()
        );
    }
    let output = run(
        Command::new("node")
            .arg(dir.join("render.mjs"))
            .current_dir(&dir)
            .env("PLAYWRIGHT_BROWSERS_PATH", dir.join(".browsers")),
        Some(input),
        Duration::from_secs(600),
    )?;
    serde_json::from_str(&output).context("Renderer returned invalid JSON")
}
