use dorm::{prelude::SqlChunk, query::Query};

use sqlformat::FormatOptions;
use sqlformat::QueryParams;
// use syntect::easy::HighlightLines;
// use syntect::highlighting::Style;
// use syntect::highlighting::ThemeSet;
// use syntect::parsing::SyntaxSet;
// use syntect::util::as_24_bit_terminal_escaped;
// use syntect::util::LinesWithEndings;

pub fn format_query(q: &Query) -> String {
    let qs = q.render_chunk().split();

    let formatted_sql = sqlformat::format(
        &qs.0.replace("{}", "?"),
        &QueryParams::Indexed(qs.1.iter().map(|x| x.to_string()).collect::<Vec<String>>()),
        FormatOptions::default(),
    );

    formatted_sql

    // let ps = SyntaxSet::load_defaults_newlines();
    // let ts = ThemeSet::load_defaults();

    // // Choose a theme
    // let theme = &ts.themes["base16-ocean.dark"];

    // // Get the syntax definition for SQL
    // let syntax = ps.find_syntax_by_extension("sql").unwrap();

    // // Create a highlighter
    // let mut h = HighlightLines::new(syntax, theme);

    // // Apply highlighting
    // let mut highlighted_sql = String::new();
    // for line in LinesWithEndings::from(&formatted_sql) {
    //     let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
    //     let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
    //     highlighted_sql.push_str(&escaped);
    // }

    // highlighted_sql
}
