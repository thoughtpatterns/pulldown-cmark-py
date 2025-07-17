class Options:
    tables: bool
    footnotes: bool
    strikethrough: bool
    tasklists: bool
    smart_punctuation: bool
    heading_attributes: bool
    yaml_style_metadata_blocks: bool
    pluses_delimited_metadata_blocks: bool
    old_footnotes: bool
    math: bool
    gfm: bool
    definition_list: bool
    superscript: bool
    subscript: bool
    wikilinks: bool
    highlight: bool

    def __init__(
        self,
        *,
        tables: bool = False,
        footnotes: bool = False,
        strikethrough: bool = False,
        tasklists: bool = False,
        smart_punctuation: bool = False,
        heading_attributes: bool = False,
        yaml_style_metadata_blocks: bool = False,
        pluses_delimited_metadata_blocks: bool = False,
        old_footnotes: bool = False,
        math: bool = False,
        gfm: bool = False,
        definition_list: bool = False,
        superscript: bool = False,
        subscript: bool = False,
        wikilinks: bool = False,
        highlight: bool = False,
    ) -> None: ...

def css(theme: str) -> str: ...
def render(
    markdown: list[str],
    options: Options | None = None,
) -> list[str]: ...

THEMES: list[str]

class PulldownCmarkError(Exception): ...
class CannotRenderMathError(PulldownCmarkError): ...
class CannotConfigMathError(PulldownCmarkError): ...
class CannotHighlightError(PulldownCmarkError): ...
class CannotGetCssError(PulldownCmarkError): ...
class UnknownLanguageError(PulldownCmarkError): ...
class UnknownThemeError(PulldownCmarkError): ...
class MissingThemeError(PulldownCmarkError): ...
