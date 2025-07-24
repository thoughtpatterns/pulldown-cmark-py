// TODO: just make the math/highlight stuff a python callback!

mod error;
mod highlight;
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
use pyo3::types::PyIterator;
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

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
