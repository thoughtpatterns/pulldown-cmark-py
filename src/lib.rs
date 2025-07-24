mod error;
mod iter;
mod options;

use crate::error::{BadCallbackError, Fatal, PulldownCmarkError};
use crate::iter::EventIter;
use crate::options::PyOptions;
use ::pulldown_cmark::{Parser, html::push_html};
use itertools::process_results;
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;

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
/// BadCallbackError
///    If a user callback fails while Markdown is parsed.
#[pyfunction]
#[pyo3(signature = (markdown, options = None))]
fn render(py: Python, markdown: &Bound<'_, PyList>, options: Option<&PyOptions>) -> PyResult<Vec<String>> {
	let default = PyOptions::default();
	let options = options.unwrap_or(&default);

	let inputs: Vec<String> = markdown
		.iter()
		.map(|wrapped| wrapped.extract())
		.collect::<PyResult<_>>()?;

	py.allow_threads(move || {
		inputs.par_iter()
			.map(|buffer| {
				let parser = Parser::new_ext(buffer, options.flags);
				let iter = EventIter::new(parser, &options.callbacks);
				let mut output = String::with_capacity(buffer.len());
				process_results(iter, |events| push_html(&mut output, events)).map(|_| output)
			})
			.collect::<Result<Vec<String>, Fatal>>()
			.map_err(PyErr::from)
	})
}

/// A configurable Python wrapper around `pulldown-cmark`.
#[pymodule]
fn pulldown_cmark(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_class::<PyOptions>()?;
	m.add("PulldownCmarkError", py.get_type::<PulldownCmarkError>())?;
	m.add("BadCallbackError", py.get_type::<BadCallbackError>())?;
	m.add_function(wrap_pyfunction!(render, m)?)?;
	Ok(())
}
