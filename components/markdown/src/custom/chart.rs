use anyhow::{Result, anyhow};
use giallo::ParsedFence;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[cfg(not(debug_assertions))]
const CACHE_FOLDER_NAME: &str = ".publish_cache";
#[cfg(not(debug_assertions))]
const DEBUG_CACHE_FOLDER_NAME: &str = ".debug_cache";

#[allow(unused_variables)]
pub fn format_chart(code: &ParsedFence, content: &str, is_publishing: bool) -> String {
    let content = content.trim();

    #[cfg(not(debug_assertions))]
    {
        let hash = super::utils::get_stable_hash(content).to_string();

        use std::{
            fs::{create_dir, read_to_string},
            path::Path,
        };

        let path = if is_publishing {
            Path::new(CACHE_FOLDER_NAME).join(format!("{hash}.svg"))
        } else {
            Path::new(DEBUG_CACHE_FOLDER_NAME).join(format!("{hash}.svg"))
        };

        if let Ok(contents) = read_to_string(&path) {
            return contents;
        }

        match convert_chart_to_svg(code, content) {
            Ok(contents) => {
                if is_publishing {
                    _ = create_dir(CACHE_FOLDER_NAME);
                } else {
                    _ = create_dir(DEBUG_CACHE_FOLDER_NAME);
                };
                _ = std::fs::write(path, &contents);
                return contents;
            }
            Err(e) => {
                return format!("<div class=\"custom-chart-error\">{}</div>", e);
            }
        }
    }

    match convert_chart_to_svg(code, content) {
        Ok(result) => result,
        Err(e) => format!("<div class=\"custom-chart-error\">{}</div>", e),
    }
}

fn convert_chart_to_svg(code: &ParsedFence, content: &str) -> Result<String> {
    let content = if code.lang == "vega" {
        convert_chart_to_svg_vega(code, content)?
    } else {
        convert_chart_to_svg_matplotlib(code, content)?
    };

    // let before_len = content.len() as isize;
    let content = post_process_svg_svgo(&content)?;
    // dbg!(&content);
    // let after_len = content.len() as isize;
    // info!("New chart is {} bytes smaller after post-processing", before_len - after_len);
    Ok(content)
}

// This was actually not great
#[allow(unused)]
fn convert_chart_to_svg_vega(_code: &ParsedFence, content: &str) -> Result<String> {
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

fn convert_chart_to_svg_matplotlib(_code: &ParsedFence, content: &str) -> Result<String> {
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

# Style
plt.style.use('dark_background')

# Don't hard-code text shapes
matplotlib.rcParams['svg.fonttype'] = 'none'

# Force transparent backgrounds for both the main figure and the plot area
matplotlib.rcParams['figure.facecolor'] = 'none'
matplotlib.rcParams['axes.facecolor'] = 'none'

# Ensure text, labels, and ticks are clean white
matplotlib.rcParams['text.color'] = '#FFFFFF'
matplotlib.rcParams['axes.labelcolor'] = '#FFFFFF'
matplotlib.rcParams['xtick.color'] = '#FFFFFF'
matplotlib.rcParams['ytick.color'] = '#FFFFFF'
matplotlib.rcParams['axes.edgecolor'] = '#444444'  # Soft gray border

# Make gridlines subtler so they don't clash on dark backgrounds
matplotlib.rcParams['grid.color'] = '#333333'
matplotlib.rcParams['grid.alpha'] = 0.5

# Enable responsive SVGs (viewBox scaling)
matplotlib.rcParams['svg.image_inline'] = True

# User-code to create plot
{}

# Render to string
svg_buffer = io.StringIO()
plt.savefig(svg_buffer, format='svg', bbox_inches='tight')
svg_data = svg_buffer.getvalue()
plt.close()

# Remove junk via regexes that we're too dumb to configure away

# Height/width
old_tag = re.search(r'<svg[^>]*>', svg_data).group(0)
new_tag = re.sub(r'\\s*(?:width|height)=\"[^\"]*\"', '', old_tag)
svg_data = svg_data.replace(old_tag, new_tag, 1)

# Metadata
svg_data = re.sub(r'<\\?xml[^>]*\\?>\\s*', '', svg_data)
svg_data = re.sub(r'<!DOCTYPE[^>]*>\\s*', '', svg_data)
svg_data = re.sub(r'<metadata>.*?</metadata>\\s*', '', svg_data, flags=re.DOTALL)

# Style stuff
svg_data = re.sub(r'\\s+id=\"[^\"]*\"', '', svg_data)

# This might be incorrect. Keeping until we verify either way.
svg_data = re.sub(r'\\s+xlink:href=\"[^\"]*\"', '', svg_data)
svg_data = re.sub(r'\\s+clip-path=\"[^\"]*\"', '', svg_data)

# Done to reduce size a smidge
font_family = re.search(r'font-family:([^;\"]+)[;\"]+', svg_data).group(0)
svg_data = re.sub(r'\\s*font-family:[^;\"]+[;\"]+', '', svg_data)
#svg_data = font_family

# Write to stdout
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
        // dbg!(&svg_output);
        Ok(svg_output)
    } else {
        let error_msg = String::from_utf8(output.stderr)?;
        Err(anyhow!(error_msg))
    }
}

#[allow(unused)]
fn post_process_svg_svgo(content: &str) -> Result<String> {
    let mut child = Command::new("npx")
        .args(&["-y", "svgo", "-"])
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

// This was... actually not that useful. Taking out for now
#[allow(unused)]
fn post_process_svg_magick(content: &str) -> Result<String> {
    let mut child = Command::new("magick")
        .args(&["-", "svg:-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // dbg!(&content);
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
