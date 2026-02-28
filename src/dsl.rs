use chumsky::{error::Rich, extra, prelude::*};
use std::ops::Range;

type PErr<'a> = extra::Err<Rich<'a, char>>;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutLine {
    pub name: String,
    pub scheme: Scheme,
    pub rest: Option<Rest>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scheme {
    SetsReps { sets: u32, reps: u32 },
    RepPercentList(Vec<RepPercent>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepPercent {
    pub reps: u32,
    pub percent: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rest {
    Seconds(u32),
    Minutes(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorInfo {
    /// Byte span into the *entire input string* passed to `parse_lines_all`.
    pub span: Range<usize>,
    pub message: String,
}

fn uint<'a>() -> impl Parser<'a, &'a str, u32, PErr<'a>> {
    text::int(10).from_str().unwrapped()
}

fn number<'a>() -> impl Parser<'a, &'a str, f32, PErr<'a>> {
    text::int(10).from_str().unwrapped()
}

fn ws<'a>() -> impl Parser<'a, &'a str, (), PErr<'a>> {
    one_of(" \t").repeated().ignored()
}

fn req_ws<'a>() -> impl Parser<'a, &'a str, (), PErr<'a>> {
    one_of(" \t").repeated().at_least(1).ignored()
}

fn name_parser<'a>() -> impl Parser<'a, &'a str, String, PErr<'a>> {
    // Everything up to ':' (trimmed). Empty-name handled in semantic validation.
    any()
        .and_is(just(':').not())
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|s| s.trim().to_string())
}

fn sets_reps_parser<'a>() -> impl Parser<'a, &'a str, Scheme, PErr<'a>> {
    uint()
        .then_ignore(ws())
        .then_ignore(just('x'))
        .then_ignore(ws())
        .then(uint())
        .map(|(sets, reps)| Scheme::SetsReps { sets, reps })
}

fn rep_percent_item_parser<'a>() -> impl Parser<'a, &'a str, RepPercent, PErr<'a>> {
    uint()
        .then_ignore(ws())
        .then_ignore(just('@'))
        .then_ignore(ws())
        .then(number())
        .then_ignore(ws())
        .then_ignore(just('%'))
        .map(|(reps, percent)| RepPercent { reps, percent })
}

fn rep_percent_list_parser<'a>() -> impl Parser<'a, &'a str, Scheme, PErr<'a>> {
    // whitespace-separated items; no commas
    rep_percent_item_parser()
        .separated_by(req_ws())
        .at_least(1)
        .collect::<Vec<_>>()
        .map(Scheme::RepPercentList)
}

fn scheme_parser<'a>() -> impl Parser<'a, &'a str, Scheme, PErr<'a>> {
    rep_percent_list_parser().or(sets_reps_parser())
}

fn rest_parser<'a>() -> impl Parser<'a, &'a str, Rest, PErr<'a>> {
    just('#')
        .ignore_then(ws())
        .ignore_then(uint())
        .then(one_of("sm"))
        .map(|(n, unit)| match unit {
            's' => Rest::Seconds(n),
            'm' => Rest::Minutes(n),
            _ => unreachable!(),
        })
}

pub fn line_parser<'a>() -> impl Parser<'a, &'a str, WorkoutLine, PErr<'a>> {
    ws().ignore_then(name_parser())
        .then_ignore(ws())
        .then_ignore(just(':'))
        .then_ignore(ws())
        .then(scheme_parser())
        .then(ws().ignore_then(rest_parser()).or_not())
        .then_ignore(ws())
        .map(|((name, scheme), rest)| WorkoutLine { name, scheme, rest })
        .then_ignore(end())
}

pub fn parse_line(input: &str) -> Result<WorkoutLine, Vec<Rich<'_, char>>> {
    line_parser().parse(input).into_result()
}

/// Parse a multi-line DSL input, continuing past invalid lines.
/// Returns:
/// - all successfully parsed lines
/// - all parse errors, each with a *global* byte span into the provided `input`
pub fn parse_lines_all(input: &str) -> (Vec<WorkoutLine>, Vec<ParseErrorInfo>) {
    let mut ok = Vec::new();
    let mut errs = Vec::new();

    // Keep byte offsets by iterating over newline-inclusive segments.
    let mut offset: usize = 0;

    for seg in input.split_inclusive('\n') {
        // `seg` includes trailing '\n' (except maybe last line)
        let has_nl = seg.ends_with('\n');
        let line = if has_nl { &seg[..seg.len() - 1] } else { seg };

        // Skip blank/whitespace-only lines but still advance offset.
        if line.trim().is_empty() {
            offset += seg.len();
            continue;
        }

        match parse_line(line) {
            Ok(w) => ok.push(w),
            Err(es) => {
                for e in es {
                    let sp = e.span();
                    let global = (offset + sp.start)..(offset + sp.end);
                    errs.push(ParseErrorInfo {
                        span: global,
                        message: e.to_string(),
                    });
                }
            }
        }

        offset += seg.len();
    }

    // Handle case where input does not end with '\n' (split_inclusive covers it),
    // but if input is empty, loop never runs.
    if input.is_empty() {
        return (ok, errs);
    }

    // If the input does not end with '\n', split_inclusive still yields the last
    // segment without '\n', and offset advances by seg.len(), so spans remain correct.

    (ok, errs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sets_reps_with_rest() {
        let x = parse_line("Back Squat: 3x5 #3m").unwrap();
        assert_eq!(x.name, "Back Squat");
    }

    #[test]
    fn parses_rep_percent_list_without_commas() {
        let x = parse_line("Bench: 8@70% 6@75% 4@80% #90s").unwrap();
        match x.scheme {
            Scheme::RepPercentList(v) => assert_eq!(v.len(), 3),
            _ => panic!("expected rep-percent list"),
        }
    }

    #[test]
    fn rejects_comma_separated_list_now() {
        assert!(parse_line("Bench: 8@70%, 6@75%, 4@80%").is_err());
    }

    #[test]
    fn continues_past_invalid_lines_and_reports_global_spans() {
        let input = "Back Squat: 3x5\nBad Line\nBench: 8@70% 6@75%\n";
        let (ok, errs) = parse_lines_all(input);

        assert_eq!(ok.len(), 2);
        assert!(!errs.is_empty());

        // The invalid line starts after "Back Squat: 3x5\n" => 16 bytes
        // "Bad Line" occupies bytes [16, 24)
        // Depending on the specific error span, it should fall within that line.
        let any_in_bad_line = errs.iter().any(|e| e.span.start >= 16 && e.span.end <= 24);
        assert!(any_in_bad_line);
    }
}
