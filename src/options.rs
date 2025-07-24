use ::pulldown_cmark::Options;
use pyo3::prelude::*;

#[derive(Default)]
pub struct Callbacks {
	pub math: Option<PyObject>,
	pub code: Option<PyObject>,
}

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
///     Skip YAML-style front matter blocks, which start with `---` and end with
///     `---` or `...`.
/// pluses_delimited_metadata_blocks [0]
///     Skip TOML-style front matter blocks, which start and end with `+++`.
/// old_footnotes [1]
///     Render vanilla-Markdown-style footnotes.
/// gfm
///     Render blockquote tags: [!NOTE], [!TIP], [!IMPORTANT], [!WARNING], and
///     [!CAUTION].
/// definition_list
///     Render `commonmark-hs/commonmark-extensions` definition lists.
/// superscript
///     Render superscript (`^text^`).
/// subscript
///     Render subscript (`~text~`).
/// wikilinks
///     Render Obsidian-style wikilinks.
/// math
///     A callback function with which to filter math delimited by `$` or `$$`,
///     of signature `def f(buffer: str, display: bool) -> str`.
/// code
///     A callback function with which to filter code, of signature
///     `def f(buffer: str, language: str | None) -> str`.
///
/// [0]: Front matter blocks are *not* parsed for data. These flags simply let
///      the parser skip them without error.
/// [1]: `pulldown-cmark` will enable `footnotes` if `old-footnotes` is true.
#[pyclass(name = "Options")]
pub struct PyOptions {
	pub flags: Options,
	pub callbacks: Callbacks,
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
		gfm = false,
		definition_list = false,
		superscript = false,
		subscript = false,
		wikilinks = false,
		math = None,
		code = None,
	))]
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
		gfm: bool,
		definition_list: bool,
		superscript: bool,
		subscript: bool,
		wikilinks: bool,
		math: Option<PyObject>,
		code: Option<PyObject>,
	) -> Self {
		let mut flags = Options::empty();

		macro_rules! flag_map {
			{ $( $switch:expr => $flag:expr),* $(,)? } => {
				$( if $switch { flags.insert($flag) } )*
			};
		}

		flag_map! {
			tables => Options::ENABLE_TABLES,
			footnotes => Options::ENABLE_FOOTNOTES,
			strikethrough => Options::ENABLE_STRIKETHROUGH,
			tasklists => Options::ENABLE_TASKLISTS,
			smart_punctuation => Options::ENABLE_SMART_PUNCTUATION,
			heading_attributes => Options::ENABLE_HEADING_ATTRIBUTES,
			yaml_style_metadata_blocks => Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
			pluses_delimited_metadata_blocks => Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS,
			old_footnotes => Options::ENABLE_OLD_FOOTNOTES,
			gfm => Options::ENABLE_GFM,
			definition_list => Options::ENABLE_DEFINITION_LIST,
			superscript => Options::ENABLE_SUPERSCRIPT,
			subscript => Options::ENABLE_SUBSCRIPT,
			wikilinks => Options::ENABLE_WIKILINKS,
			math.is_some() => Options::ENABLE_MATH,
		}

		Self {
			flags,
			callbacks: Callbacks { math, code },
		}
	}
}

impl Default for PyOptions {
	fn default() -> Self {
		Self {
			flags: Options::empty(),
			callbacks: Callbacks::default(),
		}
	}
}
