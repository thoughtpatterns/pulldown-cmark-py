# pulldown-cmark-py

An easy-to-use, configurable Python wrapper around
[pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark).

## Installation

```bash
uv add "git+https://git.sr.ht/~orchid/pulldown-cmark-py"
```

## Usage

`pulldown-cmark-py` provides just one function.

```python
def render(markdown: list[str], options: Options | None = None) -> list[str]: ...
```

The `Options` class configures callbacks and CommonMark extensions; see
`help(Options)` for details.

```python
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
```

Simple callback examples are given below.

```python
from latex2mathml.converter import convert
from pygments import highlight
from pygments.formatters import HtmlFormatter
from pygments.lexers import get_lexer_by_name, guess_lexer

def math_callback(buffer: str, display: bool) -> str:
    return convert(buffer, "display" if display else "inline")

def code_callback(buffer: str, language: str | None) -> str:
    lexer = get_lexer_by_name(language) if language else guess_lexer(buffer)
    return highlight(buffer, lexer, HtmlFormatter())
```
