use pyo3::{PyErr, create_exception, exceptions::PyException};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Fatal {
	#[error("failed to render math expression")]
	CannotRenderMath(#[from] katex::Error),

	#[error("failed to build katex configuration")]
	CannotConfigMath(#[from] katex::opts::OptsBuilderError),

	#[error("failed to highlight code block")]
	CannotHighlight,

	#[error("failed to get theme css")]
	CannotGetCss,

	#[error("failed to find language '{language}'")]
	UnknownLanguage { language: String },

	#[error("failed to find theme '{theme}'")]
	UnknownTheme { theme: String },

	#[error("bug: failed to find canonical theme '{theme}'")]
	MissingTheme { theme: String },
}

create_exception!(pulldown_cmark, PulldownCmarkError, PyException);
create_exception!(pulldown_cmark, CannotRenderMathError, PulldownCmarkError);
create_exception!(pulldown_cmark, CannotConfigMathError, PulldownCmarkError);
create_exception!(pulldown_cmark, CannotHighlightError, PulldownCmarkError);
create_exception!(pulldown_cmark, CannotGetCssError, PulldownCmarkError);
create_exception!(pulldown_cmark, UnknownLanguageError, PulldownCmarkError);
create_exception!(pulldown_cmark, UnknownThemeError, PulldownCmarkError);
create_exception!(pulldown_cmark, MissingThemeError, PulldownCmarkError);

impl From<Fatal> for PyErr {
	fn from(err: Fatal) -> PyErr {
		let msg = err.to_string();
		match err {
			Fatal::CannotRenderMath(_) => CannotRenderMathError::new_err(msg),
			Fatal::CannotConfigMath(_) => CannotConfigMathError::new_err(msg),
			Fatal::CannotHighlight => CannotHighlightError::new_err(msg),
			Fatal::CannotGetCss => CannotGetCssError::new_err(msg),
			Fatal::UnknownLanguage { .. } => UnknownLanguageError::new_err(msg),
			Fatal::UnknownTheme { .. } => UnknownThemeError::new_err(msg),
			Fatal::MissingTheme { .. } => MissingThemeError::new_err(msg),
		}
	}
}
