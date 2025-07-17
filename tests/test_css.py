"""Test the css function."""

# This was used as a sanity check when theme nicknames were added. I figured
# it'd be best to leave it just to catch simple errors in the nickname lookup.

# ruff: noqa: D101, D102, S101

from pathlib import Path

from pulldown_cmark import css


class TestCss:
    def test_css(self) -> None:
        assets = Path(__file__).parent / "assets"

        for theme in (
            "base16-eighties.dark",
            "base16-mocha.dark",
            "base16-ocean.dark",
            "base16-ocean.light",
            "inspired-github.light",
            "solarized.dark",
            "solarized.light",
        ):
            assert css(theme) == Path(assets / f"{theme}.css").read_text()
