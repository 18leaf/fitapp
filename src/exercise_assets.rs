use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use strsim::jaro_winkler;

#[derive(Debug, Clone)]
pub struct SvgAsset {
    pub file_name: String,
    pub path: PathBuf,
    pub svg: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedSvg {
    pub requested_name: String,
    pub matched_file_name: String,
    pub score: f64,
    pub svg: String,
}

pub fn load_svg_assets(dir: impl AsRef<Path>) -> io::Result<Vec<SvgAsset>> {
    let mut out = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let is_svg = path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("svg"))
            .unwrap_or(false);

        if !is_svg {
            continue;
        }

        let svg_raw = fs::read_to_string(&path)?;
        let svg = sanitize_svg(&svg_raw);
        let file_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_string();

        out.push(SvgAsset {
            file_name,
            path,
            svg,
        });
    }

    Ok(out)
}

pub fn resolve_svg<'a>(exercise_name: &str, assets: &'a [SvgAsset]) -> Option<ResolvedSvg> {
    let needle = normalize_name(exercise_name);

    let mut best: Option<(&SvgAsset, f64)> = None;

    for asset in assets {
        let stem = asset
            .path
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default();

        let candidate = normalize_name(stem);

        let score = score_match(&needle, &candidate);

        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((asset, score)),
        }
    }

    best.map(|(asset, score)| ResolvedSvg {
        requested_name: exercise_name.to_string(),
        matched_file_name: asset.file_name.clone(),
        score,
        svg: asset.svg.clone(),
    })
}
fn sanitize_svg(raw: &str) -> String {
    let mut svg = raw.trim().to_string();

    if svg.starts_with("<?xml") {
        if let Some(end) = svg.find("?>") {
            svg = svg[end + 2..].trim_start().to_string();
        }
    }

    if svg.starts_with("<!DOCTYPE") {
        if let Some(end) = svg.find('>') {
            svg = svg[end + 1..].trim_start().to_string();
        }
    }

    let width = extract_outer_svg_attr(&svg, "width");
    let height = extract_outer_svg_attr(&svg, "height");
    let has_viewbox = svg.contains("viewBox=");

    if !has_viewbox {
        if let (Some(w), Some(h)) = (width.as_deref(), height.as_deref()) {
            if let (Some(w), Some(h)) = (parse_svg_num(w), parse_svg_num(h)) {
                svg = svg.replacen(
                    "<svg",
                    &format!(r#"<svg viewBox="0 0 {} {}""#, trim_num(w), trim_num(h)),
                    1,
                );
            } else {
                svg = svg.replacen("<svg", r#"<svg viewBox="0 0 512 512""#, 1);
            }
        } else {
            svg = svg.replacen("<svg", r#"<svg viewBox="0 0 512 512""#, 1);
        }
    }

    svg = remove_outer_svg_attr(&svg, "width");
    svg = remove_outer_svg_attr(&svg, "height");

    svg
}

fn extract_outer_svg_attr(svg: &str, attr: &str) -> Option<String> {
    let svg_start = svg.find("<svg")?;
    let tag_end_rel = svg[svg_start..].find('>')?;
    let tag = &svg[svg_start..svg_start + tag_end_rel + 1];

    for quote in ['"', '\''] {
        let needle = format!(r#"{attr}={quote}"#);
        if let Some(pos) = tag.find(&needle) {
            let rest = &tag[pos + needle.len()..];
            if let Some(endq) = rest.find(quote) {
                return Some(rest[..endq].to_string());
            }
        }
    }

    None
}

fn parse_svg_num(s: &str) -> Option<f32> {
    let cleaned = s.trim().trim_end_matches("px");
    cleaned.parse::<f32>().ok()
}

fn trim_num(n: f32) -> String {
    if (n.fract()).abs() < f32::EPSILON {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}
fn remove_outer_svg_attr(svg: &str, attr: &str) -> String {
    let Some(svg_start) = svg.find("<svg") else {
        return svg.to_string();
    };

    let Some(tag_end_rel) = svg[svg_start..].find('>') else {
        return svg.to_string();
    };

    let tag_end = svg_start + tag_end_rel;
    let tag = &svg[svg_start..=tag_end];

    let mut cleaned = tag.to_string();

    for quote in ['"', '\''] {
        loop {
            let needle = format!(r#"{attr}={quote}"#);
            let Some(pos) = cleaned.find(&needle) else {
                break;
            };
            let rest = &cleaned[pos + needle.len()..];
            let Some(endq) = rest.find(quote) else {
                break;
            };
            let remove_end = pos + needle.len() + endq + 1;
            cleaned.replace_range(pos..remove_end, "");
        }
    }

    let mut out = String::with_capacity(svg.len());
    out.push_str(&svg[..svg_start]);
    out.push_str(&cleaned);
    out.push_str(&svg[tag_end + 1..]);
    out
}

fn normalize_name(s: &str) -> String {
    let lowered = s.to_ascii_lowercase();

    let mut words = Vec::new();
    let mut current = String::new();

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
        .into_iter()
        .filter(|w| !is_noise_word(w))
        .map(canonicalize_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_noise_word(word: &str) -> bool {
    matches!(
        word,
        "barbell"
            | "dumbbell"
            | "cable"
            | "machine"
            | "weighted"
            | "bodyweight"
            | "gym"
            | "exercise"
    )
}

fn canonicalize_token(word: String) -> String {
    match word.as_str() {
        "squats" => "squat".to_string(),
        "pulldowns" => "pulldown".to_string(),
        "pushups" => "pushup".to_string(),
        "pushupses" => "pushup".to_string(),
        "rows" => "row".to_string(),
        "rdl" => "romanian deadlift".to_string(),
        _ => word,
    }
}

fn score_match(a: &str, b: &str) -> f64 {
    let jw = jaro_winkler(a, b);

    let a_tokens: Vec<&str> = a.split_whitespace().collect();
    let b_tokens: Vec<&str> = b.split_whitespace().collect();

    let mut token_hits = 0.0;
    for at in &a_tokens {
        if b_tokens.iter().any(|bt| bt == at) {
            token_hits += 1.0;
        }
    }

    let token_score = if a_tokens.is_empty() {
        0.0
    } else {
        token_hits / a_tokens.len() as f64
    };

    (jw * 0.7) + (token_score * 0.3)
}
