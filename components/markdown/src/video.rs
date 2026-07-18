use crate::utils::get_stable_hash;
use anyhow::{Result, anyhow};
use giallo::ParsedFence;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;

const RESOLUTION: &str = "resolution.txt";

#[allow(unused_variables)]
pub fn format_video(
    code: &ParsedFence,
    content: &str,
    _is_publishing: bool,
    maybe_path: Option<&str>,
) -> Result<String> {
    let Some(base_path) = maybe_path else { return Err(anyhow!("Missing path")) };
    let hash = get_stable_hash(content.trim()).to_string();
    let markdown_path = Path::new(".").join("content").join(base_path);
    let Some(root) = markdown_path.parent() else { return Err(anyhow!("Missing parent")) };

    let mut title_arg = None;
    let mut path_arg = None;
    for line in content.split("\n").filter(|e| !e.is_empty()) {
        if let Some((key, value)) = line.split_once("=") {
            let key = key.trim();
            let value = value.replace("\"", "");
            let value = value.trim();

            if key == "title" {
                title_arg = Some(value.to_string());
            } else if key == "path" {
                path_arg = Some(value.to_string());
            } else {
                return Err(anyhow!("Invalid key/value pair: {line}"));
            }
        } else {
            return Err(anyhow!("Invalid key/value pair: {line}"));
        }
    }

    let Some(path_arg) = path_arg else {
        return Err(anyhow!("Missing path"));
    };
    let Some(title_arg) = title_arg else {
        return Err(anyhow!("Missing title"));
    };

    let video_path = root.join(&path_arg);
    if !video_path.exists() {
        return Err(anyhow!("Video path missing: {}", video_path.to_string_lossy()));
    }

    let Some(Some(stem)) = video_path.file_stem().map(|e| e.to_str()) else {
        return Err(anyhow!("Missing stem on {}", video_path.to_string_lossy()));
    };
    let Some(Some(extension)) = video_path.extension().map(|e| e.to_str()) else {
        return Err(anyhow!("Missing extension on {}", video_path.to_string_lossy()));
    };

    let output_path = video_path.with_file_name(format!("{}_optimized.{}", stem, extension));
    let resolution_path = video_path.with_file_name(RESOLUTION);

    if !output_path.exists() {
        // TODO: spawn this?
        // create this
        Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .args(&[
                "-c:v", "libx265", "-preset", "medium", "-crf", "28", "-c:a", "aac", "-b:a", "128k",
            ])
            .arg(output_path)
            .output()?;
        // TODO: is blocking correct?
    }

    if !resolution_path.exists() {
        let output = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=s=x:p=0",
            ])
            .arg(&video_path)
            .output()?;

        let output = String::from_utf8_lossy(&output.stdout).to_string();
        std::fs::write(&resolution_path, output.trim())?;
    }

    let resolution = read_to_string(resolution_path)?;
    let Some((left, right)) = resolution.split_once("x") else {
        return Err(anyhow!("Bad resolution: {}", resolution));
    };
    let width = left.parse::<usize>()? as f32;
    let height = right.parse::<usize>()? as f32;
    let aspect_ratio = width / height;

    let relative_path = Path::new(base_path);
    let Some(relative_path) = relative_path.parent() else { return Err(anyhow!("Missing parent")) };
    let relative_path = relative_path.join(&path_arg);
    let relative_path = relative_path.with_file_name(format!("{}_optimized.{}", stem, extension));
    let relative_path = relative_path.to_string_lossy();

    Ok(format!(
        "<p>
        <video loop autoplay muted title=\"{title_arg}\"
            style=\"aspect-ratio: {aspect_ratio}\"
        >
            <source src=\"/{relative_path}\" type=\"video/mp4\">
        </video>
    </p>"
    ))
}
