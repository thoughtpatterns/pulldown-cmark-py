use crate::error::Fatal;
use crate::options::PyOptions;
use ::pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use pyo3::prelude::*;

/// Wrapper which extends `pulldown_cmark::Parser` with extensions.
pub struct EventIter<'p, 'o> {
	parser: Parser<'p>,
	options: &'o PyOptions,
}

impl<'p, 'o> EventIter<'p, 'o> {
	pub fn new(parser: Parser<'p>, options: &'o PyOptions) -> Self {
		Self { parser, options }
	}
}

impl<'p, 'o> Iterator for EventIter<'p, 'o> {
	type Item = Result<Event<'p>, Fatal>;

	fn next(&mut self) -> Option<Self::Item> {
		let event = self.parser.next()?;

		let result = match event {
			Event::InlineMath(math) | Event::DisplayMath(math) => {
				let display = matches!(event, Event::DisplayMath(_));

				Python::with_gil(|py| {
					let result = self.options.math.unwrap().call1(py, (math.as_ref(), display));
					Ok(Event::Html(result.unwrap().extract::<String>(py).unwrap().into()))
				})
			}

			Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
				if self.options.code.is_some() =>
			{
				let mut code = String::new();
				for event in self {
					match event {
						Ok(Event::Text(text)) => code.push_str(&text),
						Ok(Event::End(TagEnd::CodeBlock)) => break,
						_ => (),
					}
				}

				Python::with_gil(|py| {
					let result = self.options.code.unwrap().call1(py, (&String::from(code), true));
					Ok(Event::Html(result.unwrap().extract(py).unwrap().into()))
				})
			}

			default => Ok(default),
		};

		Some(result)
	}
}
