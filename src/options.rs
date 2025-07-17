use ::pulldown_cmark::Options;
use pyo3::prelude::*;

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
///     Render highlighted syntax in codeblocks.
///
/// [0]: Front matter blocks are *not* parsed for data. These flags simply let
///      the parser skip them without error.
/// [1]: `pulldown-cmark` will enable `footnotes` if `old-footnotes` is true.
/// [2]: `pulldown-cmark` does not render math or highlight syntax; these are
///      extensions.
#[pyclass(name = "Options")]
#[derive(Clone, Copy)]
pub struct PyOptions {
	pub pulldown: Options,
	pub highlight: bool,
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

		for (option, switch) in [
			(Options::ENABLE_TABLES, tables),
			(Options::ENABLE_STRIKETHROUGH, strikethrough),
			(Options::ENABLE_TASKLISTS, tasklists),
			(Options::ENABLE_SMART_PUNCTUATION, smart_punctuation),
			(Options::ENABLE_HEADING_ATTRIBUTES, heading_attributes),
			(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS, yaml_style_metadata_blocks),
			(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS, pluses_delimited_metadata_blocks),
			(Options::ENABLE_MATH, math),
			(Options::ENABLE_GFM, gfm),
			(Options::ENABLE_DEFINITION_LIST, definition_list),
			(Options::ENABLE_SUPERSCRIPT, superscript),
			(Options::ENABLE_SUBSCRIPT, subscript),
			(Options::ENABLE_WIKILINKS, wikilinks),
		] {
			pulldown.set(option, switch);
		}

		/* The `ENABLE_OLD_FOOTNOTES` bitflag implies `ENABLE_FOOTNOTES`. Set them
		 * separately to not disable `ENABLE_FOOTNOTES` if `ENABLE_OLD_FOOTNOTES` is
		 * false. */
		match (old_footnotes, footnotes) {
			(true, _) => pulldown.insert(Options::ENABLE_OLD_FOOTNOTES),
			(false, true) => pulldown.insert(Options::ENABLE_FOOTNOTES),
			_ => (),
		}

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
