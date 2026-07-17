use crate::utils::get_stable_hash;
use anyhow::{Result, anyhow};
use giallo::ParsedFence;
use std::fs::{create_dir, read_to_string};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[allow(unused_variables)]
pub fn format_video(code: &ParsedFence, content: &str, _is_publishing: bool) -> String {
    let content = content.trim();
    let hash = get_stable_hash(content).to_string();
    return "VIDEO".into();

    // Path::new(CACHE_FOLDER_NAME).join(format!("{hash}.svg"))

    // if let Ok(contents) = read_to_string(&path) {
    //     return contents;
    // }

    // match convert_chart_to_svg(code, content) {
    //     Ok(contents) => {
    //         if is_publishing {
    //             _ = create_dir(CACHE_FOLDER_NAME);
    //         } else {
    //             _ = create_dir(DEBUG_CACHE_FOLDER_NAME);
    //         };
    //         _ = std::fs::write(path, &contents);
    //         return contents;
    //     }
    //     Err(e) => {
    //         return format!("<div class=\"custom-chart-error\">{}</div>", e);
    //     }
    // }
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
