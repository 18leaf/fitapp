/// This file is contains code for creating views to render on the htmx templates
use crate::dsl::{RepPercent, Rest, Scheme, WorkoutLine};

#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutViewLine {
    pub name: String,
    pub detail: WorkoutDetail,
    pub rest: Option<RestView>,
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
        }
    }
}
