use std::{fmt::Display, path::PathBuf};

use clap::Parser;
use read_fonts::{FontRef, TableProvider};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The fonts to check.
    font: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for path in args.font {
        let bytes = std::fs::read(&path).unwrap();
        let font_ref = FontRef::new(&bytes).unwrap();

        let mut diagnostics = vec![];
        for check in [check_win_ascent_and_descent] {
            let check_diagnostics = check(&font_ref);
            diagnostics.extend(check_diagnostics);
        }

        for diagnostic in diagnostics {
            println!("{}: {}", path.display(), diagnostic);
        }
    }
}

fn check_win_ascent_and_descent(font: &FontRef) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    let (Ok(os2), Ok(head)) = (font.os2(), font.head()) else {
        diagnostics.push(Diagnostic {
            level: Level::Fail,
            message: Box::new(FontReadError::TableName("OS/2 or head")),
        });
        return diagnostics;
    };

    let win_ascent: i32 = os2.us_win_ascent().into();
    let head_y_max: i32 = head.y_max().into();
    let win_ascent_max = head_y_max.checked_mul(2).unwrap();
    if win_ascent < head_y_max || win_ascent > win_ascent_max {
        diagnostics.push(Diagnostic {
            level: Level::Fail,
            message: Box::new(WinMetricsError::WinAscentOutsideExpecation {
                range: head_y_max..=win_ascent_max,
                got: win_ascent,
            }),
        })
    }

    let win_descent: i32 = os2.us_win_descent().into();
    let head_y_min: i32 = head.y_min().into();
    let head_y_min_abs = head_y_min.abs();
    let win_descent_max = head_y_min_abs.checked_mul(2).unwrap();
    if win_descent < head_y_min_abs || win_descent > win_descent_max {
        diagnostics.push(Diagnostic {
            level: Level::Fail,
            message: Box::new(WinMetricsError::WinDescentOutsideExpecation {
                range: head_y_min_abs..=win_descent_max,
                got: win_descent,
            }),
        })
    }

    diagnostics
}

#[derive(Debug)]
struct Diagnostic {
    level: Level,
    message: Box<dyn std::error::Error + Send + Sync>,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {}", self.level, self.message))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Level {
    Skip,
    Info,
    Warning,
    Fail,
}

#[derive(Debug, thiserror::Error)]
enum FontReadError {
    #[error("Cannot read {0} table")]
    TableName(&'static str),
}

#[derive(Debug, thiserror::Error)]
enum WinMetricsError {
    #[error("OS/2.usWinAscent value should be in the range [{}, {}], but got {got}", range.start(), range.end())]
    WinAscentOutsideExpecation {
        range: core::ops::RangeInclusive<i32>,
        got: i32,
    },
    #[error("OS/2.usWinDescent value should be in the range [{}, {}], but got {got}", range.start(), range.end())]
    WinDescentOutsideExpecation {
        range: core::ops::RangeInclusive<i32>,
        got: i32,
    },
}
