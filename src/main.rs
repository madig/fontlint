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
    let win_ascent_min: i32 = head.y_max().into();
    let win_ascent_max = win_ascent_min.checked_mul(2).unwrap();
    let win_ascent_range = win_ascent_min..=win_ascent_max;
    if !win_ascent_range.contains(&win_ascent) {
        diagnostics.push(Diagnostic {
            level: Level::Fail,
            message: Box::new(WinMetricsError::WinAscentOutsideExpecation {
                range: win_ascent_range,
                got: win_ascent,
            }),
        })
    }

    let win_descent: i32 = os2.us_win_descent().into();
    let win_descent_min: i32 = head.y_min().abs().into();
    let win_descent_max = win_descent_min.checked_mul(2).unwrap();
    let win_descent_range = win_descent_min..=win_descent_max;
    if !win_descent_range.contains(&win_descent) {
        diagnostics.push(Diagnostic {
            level: Level::Fail,
            message: Box::new(WinMetricsError::WinDescentOutsideExpecation {
                range: win_descent_range,
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
