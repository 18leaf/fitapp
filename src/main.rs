// src/main.rs
mod askama_template;
mod dsl;
mod validate;
mod workout_view;

use askama::Template;

use crate::{askama_template::WorkoutCardsTemplate, workout_view::WorkoutViewLine};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: {} <dsl_file> [open_id]", args[0]);
        std::process::exit(2);
    }

    let file_path = &args[1];
    let _open_id = args.get(2).map(|s| s.as_str());

    let input = std::fs::read_to_string(file_path).expect("Did Not Read File");

    let (parsed, _errs) = dsl::parse_lines_all(&input);

    // the view/stack of workouts parsed
    let view_lines: Vec<WorkoutViewLine> = parsed.into_iter().map(WorkoutViewLine::from).collect();
    let workout_view = WorkoutCardsTemplate { lines: view_lines };

    let rendered = workout_view.render();

    match rendered {
        Ok(r) => {
            print!("{}", r)
        }
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    }
    // diagnostics to stderr (keep stdout clean HTML)
    //for e in errs {
    //    eprintln!("ERR {:?}: {}", e.span, e.message);
    //}
}
