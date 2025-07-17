mod error;
mod iter;
mod options;

use crate::error::{
	CannotConfigMathError, CannotGetCssError, CannotHighlightError, CannotRenderMathError, Fatal,
	MissingThemeError, PulldownCmarkError, UnknownLanguageError, UnknownThemeError,
};
use crate::iter::EventIter;
use crate::options::PyOptions;
use ::pulldown_cmark::{Parser, html::push_html};
use itertools::process_results;
use once_cell::sync::Lazy;
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;
use std::collections::HashMap;
use syntect::{
	highlighting::ThemeSet,
	html::{ClassStyle, css_for_theme_with_class_style},
};

static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static THEME_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
	HashMap::from([
		("base16-eighties.dark", "base16-eighties.dark"),
		("base16-mocha.dark", "base16-mocha.dark"),
		("base16-ocean.dark", "base16-ocean.dark"),
		("base16-ocean.light", "base16-ocean.light"),
		("inspired-github.light", "InspiredGitHub"),
		("solarized.dark", "Solarized (dark)"),
		("solarized.light", "Solarized (light)"),
	])
});

/// Get a CSS string for a given theme.
///
/// Parameters
/// ----------
/// theme
///     The name of the theme to search for.
///
/// Returns
/// -------
/// A CSS string to highlight the given theme.
///
/// Raises
/// ------
/// CannotGetCssError
///    If the CSS string could not be assembled.
/// UnknownThemeError
///    If the theme provided could not be found.
/// MissingThemeError
///    If the theme alias failed to resolve to its canonical name, i.e., a bug,
///    but we avoid a panic to prevent a crash of the Python interpreter.
#[pyfunction]
#[pyo3(signature = (theme))]
fn css(theme: &str) -> PyResult<String> {
	let canonical = *THEME_ALIASES.get(theme).ok_or(Fatal::UnknownTheme {
		theme: String::from(theme),
	})?;

	let theme = THEMES.themes.get(canonical).ok_or_else(|| Fatal::MissingTheme {
		theme: String::from(canonical),
	})?;

	let result = css_for_theme_with_class_style(theme, ClassStyle::Spaced)
		.map_err(|e| Fatal::CannotGetCss(e.to_string()))?;

	Ok(result)
}

/// Render a list of Markdown strings into a list of HTML strings.
///
/// Parameters
/// ----------
/// markdown
///     A list of Markdown strings to render.
/// options
///     The Markdown extensions to enable.
///
/// Returns
/// -------
/// A list of HTML strings which preserves the indices of `markdown`.
///
/// Raises
/// ------
/// CannotRenderMathError
///    If a LaTeX expression cannot be rendered.
/// CannotConfigMathError
///    If the `katex` option builder fails.
/// CannotHighlightError
///    If a codeblock cannot be highlighted.
/// UnknownLanguageError
///    If an unknown language is used to open a code block.
#[pyfunction]
#[pyo3(signature = (markdown, options = None))]
fn render(py: Python, markdown: &Bound<'_, PyList>, options: Option<PyOptions>) -> PyResult<Vec<String>> {
	let options = options.unwrap_or_default();
	let inputs: Vec<String> = markdown
		.iter()
		.map(|wrapped| wrapped.extract())
		.collect::<PyResult<_>>()?;

	py.allow_threads(move || {
		inputs.par_iter()
			.map(|buffer| {
				let parser = Parser::new_ext(buffer, options.pulldown);
				let iter = EventIter::new(parser, &options);
				let mut output = String::with_capacity(buffer.len());
				process_results(iter, |events| push_html(&mut output, events)).map(|_| output)
			})
			.collect::<Result<Vec<String>, Fatal>>()
			.map_err(PyErr::from)
	})
}

/// An easy-to-use Python wrapper around `pulldown-cmark`.
#[pymodule]
fn pulldown_cmark(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	let themes = THEME_ALIASES
		.keys()
		.map(|theme| String::from(*theme))
		.collect::<Vec<String>>();

	m.add_class::<PyOptions>()?;
	m.add_function(wrap_pyfunction!(css, m)?)?;
	m.add_function(wrap_pyfunction!(render, m)?)?;
	m.add("THEMES", themes)?;
	m.add("PulldownCmarkError", py.get_type::<PulldownCmarkError>())?;
	m.add("CannotRenderMathError", py.get_type::<CannotRenderMathError>())?;
	m.add("CannotConfigMathError", py.get_type::<CannotConfigMathError>())?;
	m.add("CannotHighlightError", py.get_type::<CannotHighlightError>())?;
	m.add("CannotGetCssError", py.get_type::<CannotGetCssError>())?;
	m.add("UnknownLanguageError", py.get_type::<UnknownLanguageError>())?;
	m.add("UnknownThemeError", py.get_type::<UnknownThemeError>())?;
	m.add("MissingThemeError", py.get_type::<MissingThemeError>())?;

	Ok(())
}
