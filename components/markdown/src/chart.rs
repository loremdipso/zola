use anyhow::{Result, anyhow};
use giallo::ParsedFence;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[cfg(not(debug_assertions))]
const CACHE_FOLDER_NAME: &str = ".cache";

#[cfg(not(debug_assertions))]
const VERSION: u64 = 1;

pub fn format_chart(code: ParsedFence, content: &str) -> String {
    let content = content.trim();

    #[cfg(not(debug_assertions))]
    {
        let hash = get_stable_hash(content).to_string();

        use std::{
            fs::{create_dir, read_to_string},
            path::Path,
        };

        let path = Path::new(CACHE_FOLDER_NAME).join(format!("{hash}.svg"));
        if let Ok(contents) = read_to_string(&path) {
            return contents;
        }

        match convert_chart_to_svg(code, content) {
            Ok(result) => {
                let contents = format!("<div class=\"custom-chart\">{}</div>", result);
                _ = create_dir(CACHE_FOLDER_NAME);
                _ = std::fs::write(path, &contents);
                return contents;
            }
            Err(e) => {
                return format!("<div class=\"custom-chart-error\">{}</div>", e);
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        match convert_chart_to_svg(code, content) {
            Ok(result) => format!("<div class=\"custom-chart\">{}</div>", result),
            Err(e) => format!("<div class=\"custom-chart-error\">{}</div>", e),
        }
    }
}

// This was actually not great
#[allow(unused)]
fn convert_chart_to_svg_vega(_code: ParsedFence, content: &str) -> Result<String> {
    let mut child = Command::new("npx")
        .args(["--yes", "-p", "vega-lite", "-p", "vega", "-p", "vega-cli", "vl2svg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut stdin =
            child.stdin.take().ok_or("Failed to open stdin").map_err(|e| anyhow!("{e}"))?;
        stdin.write_all(content.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        let svg_output = String::from_utf8(output.stdout)?;
        Ok(svg_output)
    } else {
        let error_msg = String::from_utf8(output.stderr)?;
        Err(anyhow!(error_msg))
    }
}

// Let's just try matplotlib
fn convert_chart_to_svg(_code: ParsedFence, content: &str) -> Result<String> {
    let mut child = Command::new("python3")
        .args([
            "-c",
            &format!(
                "
import io
import re
import sys
import matplotlib

# Use a non-interactive backend to prevent GUI popups
matplotlib.use('Agg')

import matplotlib.pyplot as plt

# 1. Create your plot
{}

# Save the SVG into an in-memory string buffer first
svg_buffer = io.StringIO()
plt.savefig(svg_buffer, format='svg', bbox_inches='tight')
svg_data = svg_buffer.getvalue()
plt.close()

svg_data = re.sub(r'width=\"[^\"]+\"', '', svg_data)
svg_data = re.sub(r'height=\"[^\"]+\"', '', svg_data)
svg_data = re.sub(r'<\\?xml[^>]*\\?>\\s*', '', svg_data)
svg_data = re.sub(r'<!DOCTYPE[^>]*>\\s*', '', svg_data)
svg_data = re.sub(r'<metadata>.*?</metadata>\\s*', '', svg_data, flags=re.DOTALL)

sys.stdout.write(svg_data)
            ",
                content
            ),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("MPLBACKEND", "SVG")
        .spawn()?;

    {
        let mut stdin =
            child.stdin.take().ok_or("Failed to open stdin").map_err(|e| anyhow!("{e}"))?;
        stdin.write_all(content.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        let svg_output = String::from_utf8(output.stdout)?;
        Ok(svg_output)
    } else {
        let error_msg = String::from_utf8(output.stderr)?;
        Err(anyhow!(error_msg))
    }
}

// TODO: use good hash
#[cfg(not(debug_assertions))]
fn get_stable_hash(s: &str) -> u64 {
    let mut hash = 14695981039346656037_u64; // FNV offset basis
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64); // FNV prime
    }
    hash + VERSION
}
