use std::env;

use fitapp::exercise_assets::load_svg_assets;
use fitapp::render_workout_html_from_file_with_assets;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: {} <dsl_file> [open_id]", args[0]);
        std::process::exit(2);
    }

    let file_path = &args[1];
    let _open_id = args.get(2).map(|s| s.as_str());

    let assets =
        load_svg_assets("/home/leaf/Dev/projects/fitapp/assets").unwrap_or_else(|_| Vec::new());

    match render_workout_html_from_file_with_assets(file_path, &assets) {
        Ok(html) => {
            print!("{}", html);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
