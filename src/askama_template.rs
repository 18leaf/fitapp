use askama::Template;
use crate::workout_view::{WorkoutDetail, WorkoutViewLine};

#[derive(Template)]
#[template(path = "workout_cards.html")]
pub struct WorkoutCardsTemplate {
    pub lines: Vec<WorkoutViewLine>,
}
