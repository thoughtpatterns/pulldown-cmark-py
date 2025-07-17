use crate::error::Fatal;
use crate::options::PyOptions;
use ::pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use html_escape::encode_safe;
use katex::{Opts, OutputType, render_with_opts};
use once_cell::sync::Lazy;
use syntect::{
	html::{ClassStyle, ClassedHTMLGenerator},
	parsing::{SyntaxReference, SyntaxSet},
	util::LinesWithEndings,
};

static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Wrapper which extends `pulldown_cmark::Parser` with extensions.
pub struct EventIter<'p, 'o> {
	parser: Parser<'p>,
	options: &'o PyOptions,
}

impl<'p, 'o> EventIter<'p, 'o> {
	pub fn new(parser: Parser<'p>, options: &'o PyOptions) -> Self {
		Self { parser, options }
	}

	fn codeblock(parser: &mut Parser<'p>, language: &str) -> Result<Event<'p>, Fatal> {
		let mut code = String::new();
		for event in parser.by_ref() {
			match event {
				Event::Text(text) => code.push_str(&text),
				Event::End(TagEnd::CodeBlock) => break,
				_ => (),
			}
		}

		let (class, result) = if language.is_empty() {
			(String::new(), String::from(encode_safe(&code)))
		} else {
			let syntax = SYNTAXES
				.find_syntax_by_token(language)
				.ok_or_else(|| Fatal::UnknownLanguage {
					language: String::from(language),
				})?;

			(
				format!(r#" class="language-{}""#, encode_safe(language)),
				Self::codeblock_impl(&code, syntax)?,
			)
		};

		Ok(Event::Html(format!("<pre><code{class}>{result}</code></pre>").into()))
	}

	fn codeblock_impl(code: &str, syntax: &SyntaxReference) -> Result<String, Fatal> {
		let mut highlighter = ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAXES, ClassStyle::Spaced);

		for line in LinesWithEndings::from(code) {
			highlighter
				.parse_html_for_line_which_includes_newline(line)
				.map_err(|e| Fatal::CannotHighlight(e.to_string()))?;
		}

		Ok(highlighter.finalize())
	}

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
