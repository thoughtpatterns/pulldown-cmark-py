from collections.abc import Callable

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
    gfm: bool
    definition_list: bool
    superscript: bool
    subscript: bool
    wikilinks: bool
    math: Callable[[str, bool], str] | None
    code: Callable[[str, str | None], str] | None

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
        gfm: bool = False,
        definition_list: bool = False,
        superscript: bool = False,
        subscript: bool = False,
        wikilinks: bool = False,
        math: Callable[[str, bool], str] | None = None,
        code: Callable[[str, str | None], str] | None = None,
    ) -> None: ...

class PulldownCmarkError(Exception): ...
class BadCallbackError(PulldownCmarkError): ...

def render(markdown: list[str], options: Options | None = None) -> list[str]: ...
