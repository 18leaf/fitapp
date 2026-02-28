use axum::{
    Router,
    extract::{Form, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};

use fitapp::exercise_assets::{SvgAsset, load_svg_assets};

#[derive(Clone)]
struct AppState {
    assets: Arc<Vec<SvgAsset>>,
}

#[derive(Deserialize)]
struct WorkoutForm {
    dsl: String,
}
async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>fitapp</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        :root {
            --page-bg: #1A1B26;
            --panel-bg: #24283B;
            --panel-body-bg: #565F89;
            --card-bg: #414868;
            --card-open-bg: #414868;
            --card-border: #565F89;
            --adv-cell-bg: #7AA2F7;
            --adv-cell-border: #565F89;
            --text: #C3CdF8;
            --muted: #A9B1D6;
            --fallback-text: #7D86B2;
            --icon-color: #1A1B26;
            --shadow-color: rgba(26, 27, 38, 0.36);

            --radius: 22px;
            --page-width: 42ch;
            --button-height: 64px;
            --bottom-gap: 18px;
            --shadow: 0 10px 30px var(--shadow-color);
        }

        * {
            box-sizing: border-box;
        }

        html, body {
            margin: 0;
            padding: 0;
            min-height: 100%;
            background: var(--page-bg);
            color: var(--text);
            font-family: Inter, "SF Pro Text", "Helvetica Neue", Arial, sans-serif;
        }

        body {
            display: flex;
            justify-content: center;
        }

        .page {
            width: 100%;
            max-width: calc(var(--page-width) + 32px);
            padding: 24px 16px calc(var(--button-height) + 48px + var(--bottom-gap));
        }

        form {
            width: 100%;
        }

        .dsl-box {
            width: 100%;
            min-height: 70vh;
            max-width: var(--page-width);
            display: block;
            margin: 0 auto;

            border: 1px solid var(--card-border);
            border-radius: var(--radius);
            background: var(--card-bg);
            color: var(--text);
            box-shadow: var(--shadow);

            padding: 22px 20px;
            resize: vertical;
            outline: none;

            font: inherit;
            font-size: 1.18rem;
            line-height: 1.5;
            letter-spacing: 0.01em;

            white-space: pre-wrap;
            overflow-wrap: break-word;
            word-break: normal;
            tab-size: 4;

            caret-color: var(--adv-cell-bg);
        }

        .dsl-box::placeholder {
            color: var(--fallback-text);
        }

        .dsl-box:focus {
            border-color: var(--adv-cell-bg);
            box-shadow:
                0 0 0 3px rgba(122, 162, 247, 0.18),
                var(--shadow);
        }

         .tutorial {
            width: 100%;
            max-width: var(--page-width);
            margin: 18px auto 0;
            color: var(--text);
            font-size: 0.98rem;
            line-height: 1.55;
            letter-spacing: 0.01em;
        }

        .tutorial-title {
            margin: 0 0 8px 0;
            color: var(--muted);
            font-size: 0.95rem;
            font-weight: 600;
        }

        .tutorial pre {
            margin: 0;
            white-space: pre-wrap;
            overflow-wrap: break-word;
            word-break: normal;
            font: inherit;
            color: var(--text);
        }

        .tutorial .note {
            display: block;
            margin-top: 10px;
            color: var(--fallback-text);
            font-size: 0.93rem;
        }

        .submit-wrap {
            position: fixed;
            left: 50%;
            bottom: var(--bottom-gap);
            transform: translateX(-50%);
            width: 100%;
            max-width: calc(var(--page-width) + 32px);
            padding: 0 16px;
        }

        .submit-btn {
            width: 100%;
            height: var(--button-height);
            border: 1px solid var(--card-bg);
            border-radius: calc(var(--radius) - 2px);
            background: var(--card-bg);
            color: var(--text);
            cursor: pointer;

            font: inherit;
            font-size: 1rem;
            font-weight: 700;
            letter-spacing: 0.01em;

            box-shadow: var(--shadow);
        }

        .submit-btn:hover {
            filter: brightness(1.05);
        }

        .submit-btn:active {
            transform: translateY(1px);
        }

        @media (max-width: 640px) {
            :root {
                --page-width: 100%;
                --button-height: 58px;
                --radius: 18px;
            }

            .page {
                padding-top: 16px;
            }

            .dsl-box {
                min-height: 68vh;
                font-size: 1.05rem;
                padding: 18px 16px;
            }
        }
    </style>
</head>
<body>
    <div class="page">
        <form method="post" action="/render">
            <textarea
                class="dsl-box"
                name="dsl"
                placeholder="Enter workout DSL here"
                spellcheck="false"
                autofocus
            ></textarea>

            <div class="tutorial">
            <pre>
    USAGE
        pushup: 6x8
        pullup: 8@50% 5@55% 3@60%
        bench press: 8@50% 5@55% 3@60% #2m
        lat pulldown: 3x10 #1m
            </div>

            
            <div class="submit-wrap">
                <button class="submit-btn" type="submit">Build</button>
            </div>
        </form>
    </div>
</body>
</html>"#,
    )
}
async fn render(State(state): State<AppState>, Form(form): Form<WorkoutForm>) -> impl IntoResponse {
    let normalized = form.dsl.replace("\r\n", "\n").replace('\r', "\n");

    match fitapp::render_workout_html_from_str_with_assets(&normalized, &state.assets) {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>FitApp Error</title></head>
<body>
    <h1>Render Error</h1>
    <pre>{}</pre>
    <p><a href="/">Back</a></p>
</body>
</html>"#,
                err
            )),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    let assets_dir = "/home/leaf/Dev/projects/fitapp/assets";

    let assets = load_svg_assets(assets_dir).expect("failed to load SVG assets at startup");

    let state = AppState {
        assets: Arc::new(assets),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/render", post(render))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app).await.expect("server failed");
}
