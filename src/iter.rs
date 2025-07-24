use crate::error::Fatal;
use crate::options::Callbacks;
use ::pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use pyo3::prelude::*;
use std::mem::take;

#[derive(Default)]
enum State {
	#[default]
	Default,
	CodeBlock {
		buffer: String,
		language: String,
	},
}

/// Wrapper which extends `pulldown_cmark::Parser` with callbacks.
pub struct EventIter<'p, 'c> {
	state: State,
	parser: Parser<'p>,
	callbacks: &'c Callbacks,
}

impl<'p, 'c> EventIter<'p, 'c> {
	pub fn new(parser: Parser<'p>, callbacks: &'c Callbacks) -> Self {
		Self {
			parser,
			state: State::default(),
			callbacks,
		}
	}

	fn math(&self, buffer: &str, display: bool) -> Result<Event<'p>, Fatal> {
		/* `self.callbacks.math.unwrap()` is guaranteed, as this function is called
		 * only if `self.callbacks.math.is_some()`. */
		Python::with_gil(|py| {
			let result = self.callbacks.math.as_ref().unwrap().call1(py, (buffer, display));
			Ok(Event::Html(result?.extract::<String>(py)?.into()))
		})
	}

	fn code(&self, buffer: &str, language: &str) -> Result<Event<'p>, Fatal> {
		/* `self.callbacks.code.unwrap()` is guaranteed, as this function is called
		 * only if `state == State::CodeBlock`, which in turn is reached only if
		 * `self.callbacks.code.is_some()`. */
		Python::with_gil(|py| {
			let result = self.callbacks.code.as_ref().unwrap().call1(py, (buffer, language));
			Ok(Event::Html(result?.extract::<String>(py)?.into()))
		})
	}

}

impl<'p, 'c> Iterator for EventIter<'p, 'c> {
	type Item = Result<Event<'p>, Fatal>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let event = match self.parser.next() {
				Some(event) => event,
				None => {
					/* If we're in a codeblock, flush the buffer before we close the iterator. */
					if let State::CodeBlock { buffer, language } = take(&mut self.state) {
						return Some(self.code(&buffer, &language));
					} else {
						return None;
					}
				}
			};

			if let State::CodeBlock { buffer, language } = &mut self.state {
				match event {
					Event::End(TagEnd::CodeBlock) => {
						let (buffer, language) = (take(buffer), take(language));
						self.state = State::Default;
						return Some(self.code(&buffer, &language));
					}

					Event::Text(text) => {
						buffer.push_str(&text);
						continue;
					}

					_ => continue,
				}
			}

			match event {
				Event::InlineMath(math) if self.callbacks.math.is_some() => {
					return Some(self.math(math.as_ref(), false));
				}

				Event::DisplayMath(math) if self.callbacks.math.is_some() => {
					return Some(self.math(math.as_ref(), true));
				}

				Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
					if self.callbacks.code.is_some() =>
				{
					self.state = State::CodeBlock {
						buffer: String::new(),
						language: String::from(language),
					};

					continue;
				}

				default => return Some(Ok(default)),
			};
		}
	}
}
