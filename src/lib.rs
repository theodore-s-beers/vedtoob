#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::uninlined_format_args,
    clippy::missing_panics_doc
)]

pub mod app;
mod cache;
mod fetch;
pub mod nav;
pub mod ui;

use anyhow::Context;
use bat::PrettyPrinter;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn highlight(content: &str, language: &str) -> Result<String, anyhow::Error> {
    let mut output = String::new();

    PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language(language)
        .colored_output(true)
        .grid(false)
        .header(false)
        .line_numbers(false)
        .print_with_writer(Some(&mut output))
        .context("Failed to highlight with bat")?;

    Ok(output)
}

#[must_use]
pub fn pandoc_available() -> bool {
    Command::new("pandoc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn prettify(readme: &str) -> Result<String, anyhow::Error> {
    let mut child = Command::new("pandoc")
        .args(["-f", "gfm", "-t", "gfm", "--columns=80"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start pandoc process")?;

    child
        .stdin
        .as_mut()
        .context("Failed to open pandoc stdin")?
        .write_all(readme.as_bytes())?;

    let output = child
        .wait_with_output()
        .context("Failed to read pandoc output")?;

    let output_str = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(output_str)
}
