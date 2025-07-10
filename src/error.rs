use pyo3::{PyErr, create_exception, exceptions::PyException};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Fatal {
	#[error("failed to find language '{lang}'")]
	LangNotFound { lang: String },

	#[error("a call to syntect failed")]
	SyntectError(#[from] syntect::Error),

	#[error("failed to find theme '{theme}'")]
	ThemeNotFound { theme: String },
}

create_exception!(pulldown_cmark_py, PulldownCmarkError, PyException);

impl From<Fatal> for PyErr {
	fn from(err: Fatal) -> PyErr {
		PulldownCmarkError::new_err(err.to_string())
	}
}
