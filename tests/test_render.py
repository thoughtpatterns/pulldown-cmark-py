"""Test the render function."""

# ruff: noqa: D101, D102, S101

from textwrap import dedent
from typing import cast

from bs4 import BeautifulSoup
from bs4.element import NavigableString

from pulldown_cmark import Options, render


class TestRender:
    @staticmethod
    def assert_render(
        html: str,
        markdown: str,
        options: Options | None = None,
        *,
        highlight: bool = False,
    ) -> None:
        """Test the output of render() against a static string.

        Markdown is dedented, and output whitespace is normalized where allowed.
        """

        def normalize(string: str) -> str:
            soup = BeautifulSoup(string, "lxml")

            for node in soup.find_all(string=True):
                if node.parent and node.parent.name in ("pre", "code", "textare"):
                    continue  # Whitespace-sensitive tag bodies should not be stripped.

                text = NavigableString(" ".join(cast("str", node).split()))
                _ = node.replace_with(text)

            return str(soup)

        html = normalize(html)

        markdown = render([dedent(markdown)], options, highlight=highlight)[0]
        markdown = normalize(markdown)

        assert html == markdown

    def test_tables(self) -> None:
        html = """
        <table>
        <thead>
        <tr>
        <th>foo</th>
        <th>bar</th>
        </tr>
        </thead>
        <tbody>
        <tr>
        <td>baz</td>
        <td>qux</td>
        </tr>
        </tbody>
        </table>
        """

        markdown = """
        | foo | bar |
        | --- | --- |
        | baz | qux |
        """

        TestRender.assert_render(html, markdown, Options(tables=True))

    def test_footnotes(self) -> None:
        html = """
        <p>foo<sup class="footnote-reference"><a href=
        "#1">1</a></sup>bar<sup class="footnote-reference"><a href=
        "#2">2</a></sup>qux[^4]</p>
        <p>baz<sup class="footnote-reference"><a href="#3">3</a></sup></p>
        <div class="footnote-definition" id="1"><sup class=
        "footnote-definition-label">1</sup>
        <p>foo</p>
        </div>
        <div class="footnote-definition" id="2"><sup class=
        "footnote-definition-label">2</sup>
        <p>bar</p>
        </div>
        <div class="footnote-definition" id="3"><sup class=
        "footnote-definition-label">3</sup>
        <p>baz</p>
        </div>
        <p>quux</p>
        """

        markdown = """
        foo[^1] bar[^2] qux[^4]

        baz[^3]

        [^1]: foo
        [^2]: bar

        [^3]: baz

          quux
        """

        TestRender.assert_render(html, markdown, Options(footnotes=True))

    def test_strikethrough(self) -> None:
        html = """
        <p><del>foo</del></p>
        """

        markdown = """
        ~~foo~~
        """

        TestRender.assert_render(html, markdown, Options(strikethrough=True))

    def test_tasklists(self) -> None:
        html = """
        <ul>
        <li><input type="checkbox" disabled>foo</li>
        <li><input type="checkbox" checked disabled>bar</li>
        </ul>
        """

        markdown = """
        - [ ] foo
        - [x] bar
        """

        TestRender.assert_render(html, markdown, Options(tasklists=True))

    def test_smart_punctuation(self) -> None:
        html = """
        <p>‘foo’ “bar” baz–qux</p>
        """  # noqa: RUF001

        markdown = """
        'foo' "bar" baz--qux
        """

        TestRender.assert_render(html, markdown, Options(smart_punctuation=True))

    def test_heading_attributes(self) -> None:
        html = """
        <h1 id="bar" class="baz">foo</h1>
        """

        markdown = """
        # foo {#bar .baz}
        """

        TestRender.assert_render(html, markdown, Options(heading_attributes=True))

    def test_old_footnotes(self) -> None:
        html = """
        <p>foo<sup class="footnote-reference"><a href="#1">1</a></sup>
        bar<sup class="footnote-reference"><a href="#2">2</a></sup>
        qux<sup class="footnote-reference"><a href="#4">3</a></sup></p>
        <p>baz<sup class="footnote-reference"><a href="#3">4</a></sup></p>
        <div class="footnote-definition" id="1"><sup class=
        "footnote-definition-label">1</sup>
        <p>foo</p>
        </div>
        <div class="footnote-definition" id="2"><sup class=
        "footnote-definition-label">2</sup>
        <p>bar</p>
        </div>
        <div class="footnote-definition" id="3"><sup class=
        "footnote-definition-label">4</sup>
        <p>baz</p>
        </div>
        <p>quux</p>
        """

        markdown = """
        foo[^1] bar[^2] qux[^4]

        baz[^3]

        [^1]: foo
        [^2]: bar

        [^3]: baz

          quux
        """

        TestRender.assert_render(html, markdown, Options(old_footnotes=True))

    def test_math(self) -> None:
        html = """
        <p><span class="katex"><math xmlns=
        "http://www.w3.org/1998/Math/MathML">
        <semantics>
        <mrow>
        <msup>
        <mi>x</mi>
        <mn>2</mn>
        </msup>
        <mo>+</mo>
        <msup>
        <mi>y</mi>
        <mn>2</mn>
        </msup>
        <mo>=</mo>
        <mn>1</mn>
        </mrow>
        <annotation encoding="application/x-tex">x^2 + y^2 = 1</annotation>
        </semantics>
        </math></span> <span class="katex"><math xmlns=
        "http://www.w3.org/1998/Math/MathML" display="block">
        <semantics>
        <mrow>
        <msubsup>
        <mo>∫</mo>
        <mn>0</mn>
        <mn>1</mn>
        </msubsup>
        <msup>
        <mi>x</mi>
        <mn>2</mn>
        </msup>
        <mi mathvariant="normal">d</mi>
        <mi>x</mi>
        </mrow>
        <annotation encoding="application/x-tex">
        \\int_0^1 x^2 \\mathrm{d}x</annotation>
        </semantics>
        </math></span></p>
        """

        markdown = r"""
        $x^2 + y^2 = 1$
        $$\int_0^1 x^2 \mathrm{d}x$$
        """

        TestRender.assert_render(html, markdown, Options(math=True))

    def test_gfm(self) -> None:
        html = """
        <blockquote class="markdown-alert-note">
        <p>foo</p>
        </blockquote>
        """

        markdown = """
        > [!NOTE]
        > foo
        """

        TestRender.assert_render(html, markdown, Options(gfm=True))

    def test_definition_list(self) -> None:
        html = """
        <dl>
        <dt>foo</dt>
        <dd>bar</dd>
        <dt>baz</dt>
        <dd>qux</dd>
        </dl>
        """

        markdown = """
        foo
        : bar

        baz
        : qux
        """

        TestRender.assert_render(html, markdown, Options(definition_list=True))

    def test_superscript(self) -> None:
        html = """
        <p><sup>foo</sup></p>
        """

        markdown = """
        ^foo^
        """

        TestRender.assert_render(html, markdown, Options(superscript=True))

    def test_subscript(self) -> None:
        html = """
        <p><sub>foo</sub></p>
        """

        markdown = """
        ~foo~
        """

        TestRender.assert_render(html, markdown, Options(subscript=True))

    def test_wikilinks(self) -> None:
        html = """
        <p><a href="foo">foo</a></p>
        """

        markdown = """
        [[foo]]
        """

        TestRender.assert_render(html, markdown, Options(wikilinks=True))

    def test_highlight(self) -> None:
        html = """
        <pre><code class="language-rust"><span class=
        "source rust"><span class="storage type rust">let</span> x
            <span class="keyword operator rust">=</span> <span class=
        "constant numeric integer decimal rust">1</span><span class=
        "punctuation terminator rust">;</span>
        </span></code></pre>
        """

        markdown = """
        ```rust
        let x
            = 1;
        ```
        """
        TestRender.assert_render(html, markdown, highlight=True)
