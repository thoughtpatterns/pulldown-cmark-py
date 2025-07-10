mod error;

use crate::error::{Fatal, PulldownCmarkError};
use html_escape::encode_safe;
use itertools::process_results;
use lazy_static::lazy_static;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};
use pyo3::{Python, prelude::*, types::PyList, wrap_pyfunction};
use syntect::{
	highlighting::{Theme, ThemeSet},
	html::{ClassStyle, css_for_theme_with_class_style, highlighted_html_for_string},
	parsing::SyntaxSet,
};

lazy_static! {
	static ref ps: SyntaxSet = SyntaxSet::load_defaults_newlines();
	static ref ts: ThemeSet = ThemeSet::load_defaults();
}

/* See 'https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html'.
 * We do not allow configuration of metadata options. */
#[pyclass(name = "Options")]
#[derive(Clone, Copy)]
struct PyOptions {
	tables: bool,
	footnotes: bool,
	strikethrough: bool,
	tasklists: bool,
	smart_punctuation: bool,
	heading_attributes: bool,
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
	#[pyo3(text_signature = "(
		*,
		tables=False,
		footnotes=False,
		strikethrough=False,
		tasklists=False,
		smart_punctuation=False,
		heading_attributes=False,
		old_footnotes=False,
		math=False,
		gfm=False,
		definition_list=False,
		superscript=False,
		subscript=False,
		wikilinks=False,
	)")]
	fn new(
		tables: bool,
		footnotes: bool,
		strikethrough: bool,
		tasklists: bool,
		smart_punctuation: bool,
		heading_attributes: bool,
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

impl PyOptions {
	fn to_rust(self) -> Options {
		let mut result = Options::empty();
		result.set(Options::ENABLE_TABLES, self.tables);
		result.set(Options::ENABLE_FOOTNOTES, self.footnotes);
		result.set(Options::ENABLE_STRIKETHROUGH, self.strikethrough);
		result.set(Options::ENABLE_TASKLISTS, self.tasklists);
		result.set(Options::ENABLE_SMART_PUNCTUATION, self.smart_punctuation);
		result.set(Options::ENABLE_HEADING_ATTRIBUTES, self.heading_attributes);
		result.set(Options::ENABLE_OLD_FOOTNOTES, self.old_footnotes);
		result.set(Options::ENABLE_MATH, self.math);
		result.set(Options::ENABLE_GFM, self.gfm);
		result.set(Options::ENABLE_DEFINITION_LIST, self.definition_list);
		result.set(Options::ENABLE_SUPERSCRIPT, self.superscript);
		result.set(Options::ENABLE_SUBSCRIPT, self.subscript);
		result.set(Options::ENABLE_WIKILINKS, self.wikilinks);
		result
	}
}

struct EventIter<'a, 'b> {
	parser: Parser<'a>,
	theme: &'b str,
}

impl<'a, 'b> EventIter<'a, 'b> {
	pub fn new(parser: Parser<'a>, theme: &'b str) -> Self {
		Self { parser, theme }
	}

	fn codeblock(parser: &mut Parser<'a>, lang: &str, theme: &str) -> Result<Event<'a>, Fatal> {
		let mut result = String::new();
		for event in parser.by_ref() {
			match event {
				Event::Text(text) => result.push_str(&text),
				Event::End(TagEnd::CodeBlock) => break,
				_ => (),
			}
		}

		let result = EventIter::highlight(&result, lang, theme)?;
		Ok(Event::Html(result.into()))
	}

	fn highlight(code: &str, lang: &str, theme: &str) -> Result<String, Fatal> {
		let theme = lookup_theme(theme)?;
		let result = match ps.find_syntax_by_token(lang) {
			Some(syntax) => highlighted_html_for_string(code, &ps, syntax, theme)?,
			None => String::from(encode_safe(code)),
		};

		Ok(format!("<pre><code>{result}</code></pre>"))
	}
}

impl<'a, 'b> Iterator for EventIter<'a, 'b> {
	type Item = Result<Event<'a>, Fatal>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.parser.next()? {
			Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
				Some(Self::codeblock(&mut self.parser, &lang, self.theme))
			}

			default => Some(Ok(default)),
		}
	}
}

fn lookup_theme(theme: &str) -> Result<&Theme, Fatal> {
	ts.themes.get(theme).ok_or_else(|| Fatal::ThemeNotFound {
		theme: String::from(theme),
	})
}

#[pyfunction]
fn css(theme: &str) -> Result<String, Fatal> {
	let theme = lookup_theme(theme)?;
	Ok(css_for_theme_with_class_style(theme, ClassStyle::Spaced)?)
}

#[pyfunction]
#[pyo3(text_signature = "(markdown, options=None, theme=None)")]
fn to_html(
	markdown: &Bound<'_, PyList>,
	options: Option<PyOptions>,
	theme: Option<&str>,
) -> PyResult<Vec<String>> {
	let theme = theme.unwrap_or("base16-eighties.dark");
	let options = options
		.map(|py_options| py_options.to_rust())
		.unwrap_or(Options::empty());

	let mut result = Vec::with_capacity(markdown.len());

	for entry in markdown.iter() {
		let buffer: &str = entry.extract()?;
		let parser = Parser::new_ext(buffer, options);
		let iter = EventIter::new(parser, theme);
		let mut html = String::with_capacity(buffer.len());

		process_results(iter, |events| {
			html::push_html(&mut html, events);
		})?;

		result.push(html);
	}

	Ok(result)
}

#[pymodule]
fn pulldown_cmark_py(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(to_html, m)?)?;
	m.add_function(wrap_pyfunction!(css, m)?)?;
	m.add_class::<PyOptions>()?;
	m.add("PulldownCmarkError", py.get_type::<PulldownCmarkError>())?;
	Ok(())
}
