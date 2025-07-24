use pyo3::{PyErr, create_exception, exceptions::PyException};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Fatal {
	#[error("a user callback failed")]
	BadCallback(#[from] PyErr),
}

create_exception!(pulldown_cmark, PulldownCmarkError, PyException);
create_exception!(pulldown_cmark, BadCallbackError, PulldownCmarkError);

impl From<Fatal> for PyErr {
	fn from(err: Fatal) -> PyErr {
		let msg = err.to_string();
		match err {
			Fatal::BadCallback { .. } => BadCallbackError::new_err(msg),
		}
	}
}
