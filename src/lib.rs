pub mod askama_template;
pub mod dsl;
pub mod exercise_assets;
pub mod validate;
pub mod workout_view;

use askama::Template;
use std::path::Path;

use crate::askama_template::WorkoutCardsTemplate;
use crate::exercise_assets::{SvgAsset, load_svg_assets};
use crate::workout_view::{WorkoutViewLine, into_view_lines_with_assets};

pub fn parse_and_build_view_lines(input: &str, assets: &[SvgAsset]) -> Vec<WorkoutViewLine> {
    let (parsed, _errs) = dsl::parse_lines_all(input);
    into_view_lines_with_assets(parsed, assets)
}

pub fn render_workout_html_from_str_with_assets(
    input: &str,
    assets: &[SvgAsset],
) -> Result<String, askama::Error> {
    let lines = parse_and_build_view_lines(input, assets);
    let out = WorkoutCardsTemplate { lines };
    out.render()
}

pub fn render_workout_html_from_file_with_assets(
    file_path: impl AsRef<Path>,
    assets: &[SvgAsset],
) -> Result<String, Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(file_path)?;
    let html = render_workout_html_from_str_with_assets(&input, assets)?;
    Ok(html)
}

pub fn render_workout_html_from_file(
    file_path: impl AsRef<Path>,
    assets_dir: impl AsRef<Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    let assets = load_svg_assets(assets_dir)?;
    render_workout_html_from_file_with_assets(file_path, &assets)
}
