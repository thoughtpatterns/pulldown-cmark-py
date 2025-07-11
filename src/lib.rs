mod error;

use crate::error::{
	CannotConfigMathError, CannotHighlightError, CannotRenderMathError, Fatal,
	PulldownCmarkError, UnknownLanguageError, UnknownThemeError,
};
use html_escape::encode_safe;
use itertools::process_results;
use katex::{Opts, OutputType, render_with_opts};
use once_cell::sync::Lazy;
use ::pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html::push_html};
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use syntect::{
	easy::HighlightLines,
	highlighting::{Theme, ThemeSet},
	html::{IncludeBackground, append_highlighted_html_for_styled_line},
	parsing::{SyntaxReference, SyntaxSet},
	util::LinesWithEndings,
};

static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Wraps `pulldown-cmark::Options` to configure CommonMark extensions.
///
/// Parameters
/// ----------
/// tables
///     Enable GFM-style table support.
/// footnotes
///     Enable GFM-style footnotes.
/// strikethrough
///     Enable strikethrough (`~~text~~`).
/// tasklists
///     Enable task lists.
/// smart_punctuation
///     Enable smart quotes and punctuation ligatures.
/// heading_attributes
///     Enable custom IDs and classes for headings.
/// yaml_style_metadata_blocks [0]
///     Enable YAML-style front matter blocks (start with `---` and end with `---` or `...`).
/// pluses_delimited_metadata_blocks [0]
///     Enable TOML-style front matter blocks (start and end with `+++`).
/// old_footnotes [1]
///     Enable vanilla-Markdown-style footnotes.
/// math [2]
///     Enable LaTeX math rendering.
/// gfm
///     Enable blockquote tags ([!NOTE], [!TIP], [!IMPORTANT], [!WARNING], [!CAUTION]).
/// definition_list
///     Enable `commonmark-hs/commonmark-extensions` definition lists.
/// superscript
///     Enable superscript (`^text^`).
/// subscript
///     Enable subscript (`~text~`).
/// wikilinks
///     Enable Obsidian-style wikilinks.
///
/// [0]: Front matter blocks are not parsed for data. These flags simply let the
///      parser skip them without error.
/// [1]: `pulldown-cmark` will enable `footnotes` if `old-footnotes` is true.
/// [2]: `pulldown-cmark` does not render math by default; this is an extension.
#[pyclass(name = "Options")]
#[derive(Clone, Copy)]
struct PyOptions {
	tables: bool,
	footnotes: bool,
	strikethrough: bool,
	tasklists: bool,
	smart_punctuation: bool,
	heading_attributes: bool,
	yaml_style_metadata_blocks: bool,
	pluses_delimited_metadata_blocks: bool,
	old_footnotes: bool,
	math: bool,
	gfm: bool,
	definition_list: bool,
	superscript: bool,
	subscript: bool,
	wikilinks: bool,
}

#[pymethods]
impl PyOptions {
	#[new]
	#[pyo3(signature = (
		*,
		tables = false,
		footnotes = false,
		strikethrough = false,
		tasklists = false,
		smart_punctuation = false,
		heading_attributes = false,
	        yaml_style_metadata_blocks = false,
	        pluses_delimited_metadata_blocks = false,
		old_footnotes = false,
		math = false,
		gfm = false,
		definition_list = false,
		superscript = false,
		subscript = false,
		wikilinks = false,
	))]
	/// Create a new `PyOptions` (`Options` in Python) instance.
	///
	/// All options are disabled by default.
	#[allow(clippy::too_many_arguments)]
	fn new(
		tables: bool,
		footnotes: bool,
		strikethrough: bool,
		tasklists: bool,
		smart_punctuation: bool,
		heading_attributes: bool,
		yaml_style_metadata_blocks: bool,
		pluses_delimited_metadata_blocks: bool,
		old_footnotes: bool,
		math: bool,
		gfm: bool,
		definition_list: bool,
		superscript: bool,
		subscript: bool,
		wikilinks: bool,
	) -> Self {
		Self {
			tables,
			footnotes,
			strikethrough,
			tasklists,
			smart_punctuation,
			heading_attributes,
			yaml_style_metadata_blocks,
			pluses_delimited_metadata_blocks,
			old_footnotes,
			math,
			gfm,
			definition_list,
			superscript,
			subscript,
			wikilinks,
		}
	}
}

#[rustfmt::skip]
impl From<PyOptions> for Options {
	/// Convert `PyOptions` to `pulldown_cmark::Options`.
	fn from(from: PyOptions) -> Self {
		let mut result = Options::empty();

		result.set(Options::ENABLE_TABLES, from.tables);
		result.set(Options::ENABLE_STRIKETHROUGH, from.strikethrough);
		result.set(Options::ENABLE_TASKLISTS, from.tasklists);
		result.set(Options::ENABLE_SMART_PUNCTUATION, from.smart_punctuation);
		result.set(Options::ENABLE_HEADING_ATTRIBUTES, from.heading_attributes);
		result.set(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS, from.yaml_style_metadata_blocks);
		result.set(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS, from.pluses_delimited_metadata_blocks);
		result.set(Options::ENABLE_MATH, from.math);
		result.set(Options::ENABLE_GFM, from.gfm);
		result.set(Options::ENABLE_DEFINITION_LIST, from.definition_list);
		result.set(Options::ENABLE_SUPERSCRIPT, from.superscript);
		result.set(Options::ENABLE_SUBSCRIPT, from.subscript);
		result.set(Options::ENABLE_WIKILINKS, from.wikilinks);

		/* `ENABLE_OLD_FOOTNOTES` implies `ENABLE_FOOTNOTES`. Set them separately
		 * to not disable `ENABLE_FOOTNOTES` if `ENABLE_OLD_FOOTNOTES` is false. */
		if from.old_footnotes {
			result.insert(Options::ENABLE_OLD_FOOTNOTES);
		} else if from.footnotes {
			result.insert(Options::ENABLE_FOOTNOTES);
		}

		result
	}
}

/// Wrapper around `pulldown_cmark::Parser` to highlight syntax and render math.
struct EventIter<'a, 'b> {
	/// The wrapped parser.
	parser: Parser<'a>,
	/// The `syntect` to use. If `None`, code is not highlighted.
	theme: Option<&'b Theme>,
	/// Whether to render math.
	math: bool,
}

impl<'a, 'b> EventIter<'a, 'b> {
	/// Create a new `EventIter`.
	pub fn new(parser: Parser<'a>, theme: Option<&'b Theme>, math: bool) -> Self {
		Self {
			parser,
			theme,
			math,
		}
	}

	/// Handle a fenced codeblock, and highlight syntax if a language is specified.
	fn codeblock(
		parser: &mut Parser<'a>,
		language: &str,
		theme: &Theme,
	) -> Result<Event<'a>, Fatal> {
		let mut code = String::new();

		for event in parser.by_ref() {
			match event {
				Event::Text(text) => code.push_str(&text),
				Event::End(TagEnd::CodeBlock) => break,
				_ => (),
			}
		}

		let class = if language.is_empty() {
			String::new()
		} else {
			format!(r#" class="language-{}""#, encode_safe(language))
		};

		let result = match SYNTAXES.find_syntax_by_token(language) {
			Some(syntax) => EventIter::highlight(&code, syntax, theme)?,
			None => String::from(encode_safe(&code)),
		};

		let result = format!("<pre><code{class}>{result}</code></pre>");

		Ok(Event::Html(result.into()))
	}

	/// Highlight a string of code given a syntax and a theme.
	///
	/// Works similarly to `syntect::html::highlighted_html_for_string()`.
	fn highlight(code: &str, syntax: &SyntaxReference, theme: &Theme) -> Result<String, Fatal> {
		let mut highlighter = HighlightLines::new(syntax, theme);
		let mut result = String::new();

		for line in LinesWithEndings::from(code) {
			let regions = highlighter.highlight_line(line, &SYNTAXES)?;
			append_highlighted_html_for_styled_line(
				&regions,
				IncludeBackground::No,
				&mut result,
			)?;
		}

		Ok(result)
	}

	/// Render a math expression, inline or display, into MathML.
	fn math(math: &str, display: bool) -> Result<Event<'a>, Fatal> {
		let opts = Opts::builder()
			.display_mode(display)
			.output_type(OutputType::Mathml)
			.throw_on_error(true)
			.trust(false)
			.build()?;

		let result = render_with_opts(math, &opts)?;
		Ok(Event::Html(result.into()))
	}
}

impl<'a, 'b> Iterator for EventIter<'a, 'b> {
	type Item = Result<Event<'a>, Fatal>;

	/// Advance the iterator, and intercept codeblocks and math expressions.
	fn next(&mut self) -> Option<Self::Item> {
		Some(match self.parser.next()? {
			Event::InlineMath(math) if self.math => Self::math(&math, false),
			Event::DisplayMath(math) if self.math => Self::math(&math, true),

			Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
				if self.theme.is_some() =>
			{
				Self::codeblock(&mut self.parser, &language, self.theme.unwrap())
			}

			default => Ok(default),
		})
	}
}

/// Render a list of Markdown strings into a list of HTML strings.
///
/// Parameters
/// ----------
/// markdown
///     A list of Markdown strings to render.
/// options
///     The `pulldown_cmark` extensions to enable.
/// theme
///     The name of the `syntect` theme to use to highlight code. If None,
///     do not highlight code.
///
/// Returns
/// -------
/// CannotRenderMathError
///    If a LaTeX expression cannot be rendered.
/// CannotConfigMathError
///    If the `katex` option builder fails.
/// CannotHighlightError
///    If a codeblock cannot be highlighted.
/// UnknownLanguageError
///    If an unknown language is used to open a code block.
/// UnknownThemeError
///    If the specified theme cannot be found.
#[pyfunction]
#[pyo3(signature = (markdown, options = None, theme = None))]
fn render(
	markdown: &Bound<'_, PyList>,
	options: Option<PyOptions>,
	theme: Option<&str>,
) -> PyResult<Vec<String>> {
	let theme: Option<&Theme> = theme
		.map(|name| {
			THEMES.themes.get(name).ok_or_else(|| Fatal::UnknownTheme {
				theme: String::from(name),
			})
		})
		.transpose()?;

	let options = options
		.map(|py_options| py_options.into())
		.unwrap_or(Options::empty());

	let mut result = Vec::with_capacity(markdown.len());

	for entry in markdown.iter() {
		let buffer: &str = entry.extract()?;
		let parser = Parser::new_ext(buffer, options);
		let iter = EventIter::new(parser, theme, options.contains(Options::ENABLE_MATH));
		let mut html = String::with_capacity(buffer.len());

		process_results(iter, |events| {
			push_html(&mut html, events);
		})?;

		result.push(html);
	}

	Ok(result)
}

/// A Python wrapper around `pulldown-cmark` which can highlight syntax and render math.
#[rustfmt::skip]
#[pymodule]
fn pulldown_cmark(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_class::<PyOptions>()?;
	m.add_function(wrap_pyfunction!(render, m)?)?;
	m.add("PulldownCmarkError", py.get_type::<PulldownCmarkError>())?;
	m.add("CannotRenderMathError", py.get_type::<CannotRenderMathError>())?;
	m.add("CannotConfigMathError", py.get_type::<CannotConfigMathError>())?;
	m.add("CannotHighlightError", py.get_type::<CannotHighlightError>())?;
	m.add("UnknownLanguageError", py.get_type::<UnknownLanguageError>())?;
	m.add("UnknownThemeError", py.get_type::<UnknownThemeError>())?;
	Ok(())
}
