use anyhow::{Result, anyhow};
use giallo::ParsedFence;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;

const RESOLUTION: &str = "resolution.txt";

#[allow(unused_variables)]
pub fn format_image(
    code: &ParsedFence,
    content: &str,
    _is_publishing: bool,
    maybe_path: Option<&str>,
) -> Result<String> {
    // TODO: should we enable optimization, or is that too far?
    let do_optimize = false;

    let Some(base_path) = maybe_path else { return Err(anyhow!("Missing path")) };
    // let hash = get_stable_hash(content.trim()).to_string();
    let markdown_path = Path::new(".").join("content").join(base_path);
    let Some(root) = markdown_path.parent() else { return Err(anyhow!("Missing parent")) };

    let mut title_arg = None;
    let mut path_arg = None;
    let mut max_height = None;

    for line in content.split("\n").filter(|e| !e.is_empty()) {
        if let Some((key, value)) = line.split_once("=") {
            let key = key.trim();
            let value = value.replace("\"", "");
            let value = value.trim();

            if key == "title" {
                title_arg = Some(value.to_string());
            } else if key == "path" {
                path_arg = Some(value.to_string());
            } else if key == "max_height" {
                if let Ok(value) = value.parse::<usize>() {
                    max_height = Some(value);
                } else {
                    return Err(anyhow!("Invalid max_height: {value}"));
                }
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

    let image_path = root.join(&path_arg);
    if !image_path.exists() {
        return Err(anyhow!("Image path missing: {}", image_path.to_string_lossy()));
    }

    let Some(Some(stem)) = image_path.file_stem().map(|e| e.to_str()) else {
        return Err(anyhow!("Missing stem on {}", image_path.to_string_lossy()));
    };
    let Some(Some(extension)) = image_path.extension().map(|e| e.to_str()) else {
        return Err(anyhow!("Missing extension on {}", image_path.to_string_lossy()));
    };

    let output_path = if do_optimize {
        image_path.with_file_name(format!("{}_optimized.{}", stem, extension))
    } else {
        image_path.clone()
    };

    let resolution_path = image_path.with_file_name(RESOLUTION);

    if !output_path.exists() {
        if do_optimize {
            panic!();
            // TODO: spawn this?
            // create this
            // Command::new("ffmpeg")
            //     .arg("-i")
            //     .arg(&image_path)
            //     .args(&[
            //         "-c:v", "libx265", "-preset", "medium", "-crf", "28", "-c:a", "aac", "-b:a",
            //         "128k",
            //     ])
            //     .arg(output_path)
            //     .output()?;
            // TODO: is blocking correct?
        } else {
            return Err(anyhow!("Video file doesn't exist: {}", output_path.to_string_lossy()));
        }
    }

    let resolution = if !resolution_path.exists() {
        let output =
            Command::new("identify").args(&["-format", "'%wx%h'"]).arg(&image_path).output()?;

        let output = String::from_utf8_lossy(&output.stdout).to_string();
        let output = output.trim();
        std::fs::write(&resolution_path, output)?;
        output.into()
    } else {
        read_to_string(resolution_path)?
    };

    let Some((left, right)) = resolution.split_once("x") else {
        return Err(anyhow!("Bad resolution: {}", resolution));
    };
    let width = left.parse::<usize>()? as f32;
    let height = right.parse::<usize>()? as f32;
    let aspect_ratio = width / height;

    let relative_path = Path::new(base_path);
    let Some(relative_path) = relative_path.parent() else { return Err(anyhow!("Missing parent")) };
    let mut relative_path = relative_path.join(&path_arg);
    if do_optimize {
        relative_path = relative_path.with_file_name(format!("{}_optimized.{}", stem, extension));
    }

    let relative_path = relative_path.to_string_lossy();

    // Do I need this, actually?
    todo!();

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

/*
{% macro responsive_image(path, alt, title="", class=false, is_preemptive=false, new_width=0, max_height=0) %}
    <picture>
      {% set width = 0 %}
      {% set height = 0 %}

      {% if new_width > 0 and path is matching("(jpg|jpeg)$") %}
        {# Use a reasonable quality for webp #}
        {% set new_image = resize_image(path=path, width=new_width, op="fit_width", format="webp", quality=75) %}
        {% set width = new_image.width %}
        {% set height = new_image.height %}
        <source srcset="{{ self::rel_url(path=new_image.url) | safe }}" type="image/webp">

        {# TODO: add avif when that's available #}

        {% set backup_image = resize_image(path=path, width=new_width, op="fit_width") %}
        {% set path = self::rel_url(path=backup_image.url) %}
      {% elif new_width > 0 and path is matching("(png|webp)$") %}
        {% set new_image = resize_image(path=path, width=new_width, op="fit_width", format="webp") %}
        {% set width = new_image.width %}
        {% set height = new_image.height %}
        <source srcset="{{ self::rel_url(path=new_image.url) | safe }}" type="image/webp">

        {# TODO: add avif when that's available #}

        {% set backup_image = resize_image(path=path, width=new_width, op="fit_width") %}
        {% set path = self::rel_url(path=backup_image.url) %}
      {% else %}
        {% set meta = get_image_metadata(path=path) %}
        {% set width = meta.width %}
        {% set height = meta.width %}
      {% endif %}

      <img src="{{ path | safe }}"
        alt="{{ alt }}"
        {% if title %}title="{{ title }}"{% endif %}
        {% if class %}class="{{ class }}"{% endif %}
        {% if not is_preemptive %}loading="lazy" decoding="async"{% endif %}
        {% if max_height %}style="max-height: {{max_height}}px; max-width: 100%"{% endif %}
        width="{{ width }}" height="{{ height }}" />
    </picture>
{% endmacro responsive_image %}
*/
