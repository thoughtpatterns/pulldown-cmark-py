mod error;
mod options;

use crate::error::{
	CannotConfigMathError, CannotGetCssError, CannotHighlightError, CannotRenderMathError, Fatal,
	MissingThemeError, PulldownCmarkError, UnknownLanguageError, UnknownThemeError,
};
use ::pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html::push_html};
use html_escape::encode_safe;
use itertools::process_results;
use katex::{Opts, OutputType, render_with_opts};
use once_cell::sync::Lazy;
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use std::collections::HashMap;
use syntect::{
	highlighting::ThemeSet,
	html::{ClassStyle, ClassedHTMLGenerator, css_for_theme_with_class_style},
	parsing::{SyntaxReference, SyntaxSet},
	util::LinesWithEndings,
};

static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Provide a uniform theme name style.
static THEME_NICKNAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
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

/// Wraps `pulldown-cmark::Options` to configure CommonMark extensions.
///
/// Parameters
/// ----------
/// tables
///     Render GFM-style tables.
/// footnotes
///     Render GFM-style footnotes.
/// strikethrough
///     Render strikethrough (`~~text~~`).
/// tasklists
///     Render task lists.
/// smart_punctuation
///     Render smart quotes and punctuation ligatures.
/// heading_attributes
///     Render custom IDs and classes for headings.
/// yaml_style_metadata_blocks [0]
///     Skip YAML-style front matter blocks, which start with `---` and end with `---` or `...`.
/// pluses_delimited_metadata_blocks [0]
///     Skip TOML-style front matter blocks, which start and end with `+++`.
/// old_footnotes [1]
///     Render vanilla-Markdown-style footnotes.
/// math [2]
///     Render LaTeX, delimited with `$` for inline or `$$` for display.
/// gfm
///     Render blockquote tags: [!NOTE], [!TIP], [!IMPORTANT], [!WARNING], and [!CAUTION].
/// definition_list
///     Render `commonmark-hs/commonmark-extensions` definition lists.
/// superscript
///     Render superscript (`^text^`).
/// subscript
///     Render subscript (`~text~`).
/// wikilinks
///     Render Obsidian-style wikilinks.
/// highlight [2]
///     Highlight syntax in codeblocks.
///
/// [0]: Front matter blocks are *not* parsed for data. These flags simply let
///      the parser skip them without error.
/// [1]: `pulldown-cmark` will enable `footnotes` if `old-footnotes` is true.
/// [2]: `pulldown-cmark` does not render math or highlight syntax; these are
///      extensions.
#[pyclass(name = "Options")]
#[derive(Clone, Copy)]
struct PyOptions {
	pulldown: Options,
	highlight: bool,
}

#[pymethods]
impl PyOptions {
	/// Create a new `PyOptions` (`Options` in Python) instance.
	///
	/// All options are disabled by default.
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
		highlight = false,
	))]
	#[allow(clippy::too_many_arguments)]
	#[rustfmt::skip]
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
		highlight: bool,
	) -> Self {
		let mut pulldown = Options::empty();

		pulldown.set(Options::ENABLE_TABLES, tables);
		pulldown.set(Options::ENABLE_STRIKETHROUGH, strikethrough);
		pulldown.set(Options::ENABLE_TASKLISTS, tasklists);
		pulldown.set(Options::ENABLE_SMART_PUNCTUATION, smart_punctuation);
		pulldown.set(Options::ENABLE_HEADING_ATTRIBUTES, heading_attributes);
		pulldown.set(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS, yaml_style_metadata_blocks);
		pulldown.set(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS, pluses_delimited_metadata_blocks);
		pulldown.set(Options::ENABLE_MATH, math);
		pulldown.set(Options::ENABLE_GFM, gfm);
		pulldown.set(Options::ENABLE_DEFINITION_LIST, definition_list);
		pulldown.set(Options::ENABLE_SUPERSCRIPT, superscript);
		pulldown.set(Options::ENABLE_SUBSCRIPT, subscript);
		pulldown.set(Options::ENABLE_WIKILINKS, wikilinks);

		/* `ENABLE_OLD_FOOTNOTES` implies `ENABLE_FOOTNOTES`. Set them separately
		 * to not disable `ENABLE_FOOTNOTES` if `ENABLE_OLD_FOOTNOTES` is false. */
		if old_footnotes { pulldown.insert(Options::ENABLE_OLD_FOOTNOTES); }
		else if footnotes { pulldown.insert(Options::ENABLE_FOOTNOTES); }

		Self { pulldown, highlight }
	}
}

impl PyOptions {
	fn empty() -> Self {
		Self {
			pulldown: Options::empty(),
			highlight: false,
		}
	}
}

impl Default for PyOptions {
	fn default() -> Self {
		Self::empty()
	}
}

/// Wrapper around `pulldown_cmark::Parser` to highlight syntax and render math.
struct EventIter<'p, 'o> {
	parser: Parser<'p>,
	options: &'o PyOptions,
}

impl<'p, 'o> EventIter<'p, 'o> {
	/// Create a new `EventIter`.
	pub fn new(parser: Parser<'p>, options: &'o PyOptions) -> Self {
		Self { parser, options }
	}

	/// Handle a fenced codeblock: highlight syntax if a language is specified, else escape HTML.
	fn codeblock(parser: &mut Parser<'p>, language: &str) -> Result<Event<'p>, Fatal> {
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
			Some(syntax) => EventIter::codeblock_impl(&code, syntax)?,
			None => String::from(encode_safe(&code)),
		};

		Ok(Event::Html(format!("<pre><code{class}>{result}</code></pre>").into()))
	}

	/// Highlight a string of code, given a syntax.
	fn codeblock_impl(code: &str, syntax: &SyntaxReference) -> Result<String, Fatal> {
		let mut highlighter = ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAXES, ClassStyle::Spaced);

		for line in LinesWithEndings::from(code) {
			highlighter
				.parse_html_for_line_which_includes_newline(line)
				.map_err(|_| Fatal::CannotHighlight)?;
		}

		Ok(highlighter.finalize())
	}

	/// Render a math expression, inline or display, into MathML.
	fn math(math: &str, display: bool) -> Result<Event<'p>, Fatal> {
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

impl<'p, 'o> Iterator for EventIter<'p, 'o> {
	type Item = Result<Event<'p>, Fatal>;

	/// Advance the iterator, and intercept codeblocks and math expressions.
	fn next(&mut self) -> Option<Self::Item> {
		Some(match self.parser.next()? {
			Event::InlineMath(math) if self.options.pulldown.contains(Options::ENABLE_MATH) => {
				Self::math(&math, false)
			}

			Event::DisplayMath(math) if self.options.pulldown.contains(Options::ENABLE_MATH) => {
				Self::math(&math, true)
			}

			Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) if self.options.highlight => {
				Self::codeblock(&mut self.parser, &language)
			}

			default => Ok(default),
		})
	}
}

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
///    If the theme nickname failed to resolve to its canonical name, i.e., a bug,
///    but we avoid a panic to prevent a crash of the Python interpreter.
#[pyfunction]
#[pyo3(signature = (theme))]
fn css(theme: String) -> PyResult<String> {
	let canonical = *THEME_NICKNAMES
		.get(theme.as_str())
		.ok_or(Fatal::UnknownTheme { theme })?;

	let theme = THEMES.themes.get(canonical).ok_or_else(|| Fatal::MissingTheme {
		theme: canonical.into(),
	})?;

	Ok(css_for_theme_with_class_style(theme, ClassStyle::Spaced).map_err(|_| Fatal::CannotGetCss)?)
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
fn render(markdown: &Bound<'_, PyList>, options: Option<PyOptions>) -> PyResult<Vec<String>> {
	let options = options.unwrap_or_default();
	let mut result = Vec::with_capacity(markdown.len());

	for entry in markdown.iter() {
		let buffer: &str = entry.extract()?;
		let parser = Parser::new_ext(buffer, options.pulldown);
		let iter = EventIter::new(parser, &options);

		let mut html = String::with_capacity(buffer.len());
		process_results(iter, |events| {
			push_html(&mut html, events);
		})?;

		result.push(html);
	}

	Ok(result)
}

/// A Python wrapper around `pulldown-cmark` which can highlight syntax and render math.
#[pymodule]
fn pulldown_cmark(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	/* Constant attribute to permit theme configuration validation. */
	let themes = THEME_NICKNAMES
		.keys()
		.map(|x| String::from(*x))
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
