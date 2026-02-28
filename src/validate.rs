use crate::dsl::{RepPercent, Rest, Scheme, WorkoutLine};

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

pub fn validate_line(line: &WorkoutLine) -> Vec<ValidationError> {
    let mut out = Vec::new();

    if line.name.trim().is_empty() {
        out.push(err("name cannot be empty"));
    }

    match &line.scheme {
        Scheme::SetsReps { sets, reps } => {
            if *sets == 0 {
                out.push(err("sets must be > 0"));
            }
            if *reps == 0 {
                out.push(err("reps must be > 0"));
            }
        }
        Scheme::RepPercentList(items) => {
            if items.is_empty() {
                out.push(err("rep-percent list cannot be empty"));
            }

            for (idx, RepPercent { reps, percent }) in items.iter().enumerate() {
                if *reps == 0 {
                    out.push(err(format!("item {} reps must be > 0", idx + 1)));
                }
                if *percent <= 0.0 {
                    out.push(err(format!("item {} percent must be > 0", idx + 1)));
                }
                if *percent > 150.0 {
                    out.push(err(format!(
                        "item {} percent seems too high (>150%)",
                        idx + 1
                    )));
                }
            }
        }
    }

    if let Some(rest) = &line.rest {
        match rest {
            Rest::Seconds(s) if *s == 0 => out.push(err("rest seconds must be > 0")),
            Rest::Minutes(m) if *m == 0 => out.push(err("rest minutes must be > 0")),
            _ => {}
        }
    }

    out
}

fn err(msg: impl Into<String>) -> ValidationError {
    ValidationError {
        message: msg.into(),
    }
}
