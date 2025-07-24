use once_cell::sync::Lazy;
use pyo3::{
	prelude::*,
	sync::GILOnceCell,
	types::{PyModule, PyType},
};
use std::collections::HashSet;

static HTML_FORMATTER: GILOnceCell<Py<PyType>> = GILOnceCell::new();

static PYGMENTS: Lazy<PyResult<Py<PyModule>>> =
	Lazy::new(|| Python::with_gil(|py| PyModule::import(py, "pygments")?.extract()));

static FORMATTERS: Lazy<PyResult<Py<PyModule>>> =
	Lazy::new(|| Python::with_gil(|py| PyModule::import(py, "pygments.formatters")?.extract()));

static STYLES: Lazy<PyResult<Py<PyModule>>> =
	Lazy::new(|| Python::with_gil(|py| PyModule::import(py, "pygments.styles")?.extract()));

static THEMES: Lazy<PyResult<HashSet<String>>> = Lazy::new(|| {
	Python::with_gil(|py| {
		PyModule::import(py, "pygments.styles")?
			.getattr("STYLE_MAP")?
			.getattr("keys")?
			.call0()?
			.extract()
	})
});

/// Wraps `pygments.HtmlFormatter` to configure syntax highlighting.
///
/// Parameters
/// ----------
/// style
///     The highlight style to use.
/// noclasses
///     Use inline styles, rather than CSS classes, for `<span>` tags (and line
///     numbers, if enabled).
/// classprefix
///     A string to prepend to all token type CSS classes.
/// cssclass
///     CSS class for the outer `<div>` tag.
/// cssstyles
///     Inline CSS styles for the outer `<div>` tag.
/// prestyles
///     Inline CSS styles for the outer `<pre>` tag.
/// linenos
///     If `"table"`, place line numbers into a table.
///     If `"inline"`, simply place numbers into the `<pre>` tag.
///     If `None`, do not output line numbers.
/// hl_lines
///     A list of lines to be highlighted; 1-indexed, i.e., regardless of
///     `linenostart`.
/// linenostart
///     The number to place on the first line.
/// linenostep
///     If set to some n > 1, only print each nth line number.
/// linenospecial
///     If set to some n > 0, each nth line number is given the CSS class
///     `"special"`.
/// nobackground
///     Do not output the background color for the outer element.
/// lineseparator
///     The string to output between lines of code; the default is `"\n"`.
/// lineanchors
///     If set to some string s, wrap each line in an anchor tag witn an `id`
///     and `name` of `s-linenumber`.
/// linespans
///     If set to some string s, wrap each line in a `<span>` tag with an `id`
///     of `s-linenumber`.
/// anchorlinenos
///     Wrap line numbers in `<a>` tags.
/// filename
///     A string with which to generate a filename to render `<pre>` blocks.
/// wrapcode
///     Wrap the code within `<pre>` blocks with `<code>`.
///
/// The `cssfile`, `debug_token_types`, `full`, `noclobber_cssfile`, `cssfile`,
/// `nowrap`, `tagsfile`, `tagurlformat`, and `title` tags are omitted here.
#[pyclass(name = "HighlightOptions")]
#[derive(Clone)]
pub struct PyHighlightOptions {
	style: Option<String>,
	noclasses: bool,
	classprefix: Option<String>,
	cssclass: Option<String>,
	cssstyles: Option<String>,
	prestyles: Option<String>,
	linenos: Option<String>,
	hl_lines: Option<Vec<usize>>,
	linenostart: Option<usize>,
	linenostep: Option<usize>,
	linenospecial: Option<usize>,
	nobackground: bool,
	lineseparator: Option<String>,
	lineanchors: Option<String>,
	linespans: Option<String>,
	anchorlinenos: bool,
	filename: Option<String>,
	wrapcode: bool,
}

#[pymethods]
impl PyHighlightOptions {
	/// Create a new `PyHighlightOptions` (`HighlightOptions` in Python) instance.
	///
	/// All options are disabled, or None, by default.
	#[new]
	#[pyo3(signature = (
		*,
		style = None,
		noclasses = false,
		classprefix = None,
		cssclass = None,
		cssstyles = None,
		prestyles = None,
		linenos = None,
		hl_lines = None,
		linenostart = None,
		linenostep = None,
		linenospecial = None,
		nobackground = false,
		lineseparator = None,
		lineanchors = None,
		linespans = None,
		anchorlinenos = false,
		filename = None,
		wrapcode = false,

	))]
	#[allow(clippy::too_many_arguments)]
	fn new(
		style: Option<String>,
		noclasses: bool,
		classprefix: Option<String>,
		cssclass: Option<String>,
		cssstyles: Option<String>,
		prestyles: Option<String>,
		linenos: Option<String>, /* As with Pygments, any Some value except `"inline"` gives `"table"`. */
		hl_lines: Option<Vec<usize>>,
		linenostart: Option<usize>,
		linenostep: Option<usize>,
		linenospecial: Option<usize>,
		nobackground: bool,
		lineseparator: Option<String>,
		lineanchors: Option<String>,
		linespans: Option<String>,
		anchorlinenos: bool,
		filename: Option<String>,
		wrapcode: bool,
	) -> Self {
		Self {
			style,
			noclasses,
			classprefix,
			cssclass,
			cssstyles,
			prestyles,
			linenos,
			hl_lines,
			linenostart,
			linenostep,
			linenospecial,
			nobackground,
			lineseparator,
			lineanchors,
			linespans,
			anchorlinenos,
			filename,
			wrapcode,
		}
	}
}

impl PyHighlightOptions {
	pub fn to_python<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {}
}

impl Default for PyHighlightOptions {
	fn default() -> Self {
		Self {
			style: None,
			noclasses: false,
			classprefix: None,
			cssclass: None,
			cssstyles: None,
			prestyles: None,
			linenos: None,
			hl_lines: None,
			linenostart: None,
			linenostep: None,
			linenospecial: None,
			nobackground: false,
			lineseparator: None,
			lineanchors: None,
			linespans: None,
			anchorlinenos: false,
			filename: None,
			wrapcode: false,
		}
	}
}
