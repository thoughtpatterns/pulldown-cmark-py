# pulldown-cmark-py

An easy-to-use Python wrapper around
[pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark).

## Installation

```bash
uv tool install "git+https://git.sr.ht/~orchid/pulldown-cmark-py"
```

## Usage

`pulldown-cmark-py` provides just two functions:
- `render` maps a list of Markdown strings to a list of HTML strings.
- `css` returns a CSS string for a given theme, which contains color
  definitions for highlighted codeblocks. A list of the available themes
  is available in the module-level constant `THEMES`.

The `Options` class is provided to configure `render`.

## To-do

- Provide full configuration for `katex`, which would provide macro support,
  safety control, etc.
- Provide full configuration for `syntect`, which would allow inline color CSS
  and CSS class namespaces.
