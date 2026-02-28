use crate::dsl::{RepPercent, Rest, Scheme, WorkoutLine};
use crate::exercise_assets::{SvgAsset, resolve_svg};

#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutViewLine {
    pub name: String,
    pub detail: WorkoutDetail,
    pub rest: Option<RestView>,

    pub svg_inline: Option<String>,
    pub matched_asset_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkoutDetail {
    Simple { sets: u32, reps: u32 },
    Advanced { items: Vec<RepPercentView> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepPercentView {
    pub reps: u32,
    pub percent: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestView {
    pub value: u32,
    pub unit: RestUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestUnit {
    Seconds,
    Minutes,
}

impl RestView {
    pub fn display(&self) -> String {
        match self.unit {
            RestUnit::Seconds => format!("{}s", self.value),
            RestUnit::Minutes => format!("{}m", self.value),
        }
    }
}

impl From<RepPercent> for RepPercentView {
    fn from(value: RepPercent) -> Self {
        Self {
            reps: value.reps,
            percent: value.percent,
        }
    }
}

impl From<Rest> for RestView {
    fn from(value: Rest) -> Self {
        match value {
            Rest::Seconds(value) => Self {
                value,
                unit: RestUnit::Seconds,
            },
            Rest::Minutes(value) => Self {
                value,
                unit: RestUnit::Minutes,
            },
        }
    }
}

impl From<Scheme> for WorkoutDetail {
    fn from(value: Scheme) -> Self {
        match value {
            Scheme::SetsReps { sets, reps } => Self::Simple { sets, reps },
            Scheme::RepPercentList(items) => Self::Advanced {
                items: items.into_iter().map(RepPercentView::from).collect(),
            },
        }
    }
}

impl From<WorkoutLine> for WorkoutViewLine {
    fn from(value: WorkoutLine) -> Self {
        Self {
            name: value.name,
            detail: value.scheme.into(),
            rest: value.rest.map(RestView::from),
            svg_inline: None,
            matched_asset_name: None,
        }
    }
}

impl WorkoutViewLine {
    pub fn from_with_assets(value: WorkoutLine, assets: &[SvgAsset]) -> Self {
        let resolved = resolve_svg(&value.name, assets);

        Self {
            name: value.name,
            detail: value.scheme.into(),
            rest: value.rest.map(RestView::from),
            svg_inline: resolved.as_ref().map(|r| r.svg.clone()),
            matched_asset_name: resolved.map(|r| r.matched_file_name),
        }
    }
}

pub fn into_view_lines_with_assets(
    lines: Vec<WorkoutLine>,
    assets: &[SvgAsset],
) -> Vec<WorkoutViewLine> {
    lines
        .into_iter()
        .map(|line| WorkoutViewLine::from_with_assets(line, assets))
        .collect()
}
