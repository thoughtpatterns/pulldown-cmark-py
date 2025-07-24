"""Test the render function."""

# ruff: noqa: D101, D102, S101

from textwrap import dedent
from typing import cast

from bs4 import BeautifulSoup
from bs4.element import NavigableString
from latex2mathml.converter import convert as to_mathml
from pygments import highlight  # pyright: ignore[reportUnknownVariableType]
from pygments.formatters import HtmlFormatter
from pygments.lexers import get_lexer_by_name, guess_lexer

from pulldown_cmark import Options, render


class TestRender:
    @staticmethod
    def assert_render(
        html: str,
        markdown: str,
        options: Options | None = None,
    ) -> None:
        """Test the output of render() against a static string.

        Markdown is dedented, and output whitespace is normalized where allowed.
        """

        def normalize(string: str) -> str:
            soup = BeautifulSoup(string, "lxml")

            for node in soup.find_all(string=True):
                if node.parent and node.parent.name in ("pre", "code", "textarea"):
                    continue  # Whitespace-sensitive tag bodies should not be stripped.

                text = NavigableString(" ".join(cast("str", node).split()))
                _ = node.replace_with(text)

            return str(soup)

        html = dedent(html)
        html = normalize(html)

        markdown = dedent(markdown)
        markdown = render([markdown], options)[0]
        markdown = normalize(markdown)

        assert html == markdown

    @staticmethod
    def math_callback(buffer: str, display: bool, /) -> str:  # noqa: FBT001
        return to_mathml(buffer, "display" if display else "inline")

    @staticmethod
    def code_callback(buffer: str, language: str | None, /) -> str:
        lexer = get_lexer_by_name(language) if language else guess_lexer(buffer)
        return highlight(buffer, lexer, HtmlFormatter())

    def test_tables(self) -> None:
        html = """
        <table>
          <thead>
            <tr>
              <th>
                foo
              </th>
              <th>
                bar
              </th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                baz
              </td>
              <td>
                qux
              </td>
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
        <p>
          foo
          <sup class="footnote-reference">
            <a href="#1">
              1
            </a>
          </sup>
          bar
          <sup class="footnote-reference">
            <a href="#2">
              2
            </a>
          </sup>
          qux[^4]
        </p>
        <p>
          baz
          <sup class="footnote-reference">
            <a href="#3">
              3
            </a>
          </sup>
        </p>
        <div class="footnote-definition" id="1">
          <sup class="footnote-definition-label">
            1
          </sup>
          <p>
            foo
          </p>
        </div>
        <div class="footnote-definition" id="2">
          <sup class="footnote-definition-label">
            2
          </sup>
          <p>
            bar
          </p>
        </div>
        <div class="footnote-definition" id="3">
          <sup class="footnote-definition-label">
            3
          </sup>
          <p>
            baz
          </p>
        </div>
        <p>
          quux
        </p>
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
        <p>
          <del>
            foo
          </del>
        </p>
        """

        markdown = """
        ~~foo~~
        """

        TestRender.assert_render(html, markdown, Options(strikethrough=True))

    def test_tasklists(self) -> None:
        html = """
        <ul>
          <li>
            <input disabled type="checkbox">
            foo
          </li>
          <li>
            <input checked disabled type="checkbox">
            bar
          </li>
        </ul>
        """

        markdown = """
        - [ ] foo
        - [x] bar
        """

        TestRender.assert_render(html, markdown, Options(tasklists=True))

    def test_smart_punctuation(self) -> None:
        html = """
        <p>
          &lsquo;foo&rsquo; &ldquo;bar&rdquo; baz&ndash;qux
        </p>
        """

        markdown = """
        'foo' "bar" baz--qux
        """

        TestRender.assert_render(html, markdown, Options(smart_punctuation=True))

    def test_heading_attributes(self) -> None:
        html = """
        <h1 class="baz" id="bar">
          foo
        </h1>
        """

        markdown = """
        # foo {#bar .baz}
        """

        TestRender.assert_render(html, markdown, Options(heading_attributes=True))

    def test_old_footnotes(self) -> None:
        html = """
                <p>
                  foo
                  <sup class="footnote-reference">
                    <a href="#1">
                      1
                    </a>
                  </sup>
                  bar
                  <sup class="footnote-reference">
                    <a href="#2">
                      2
                    </a>
                  </sup>
                  qux
                  <sup class="footnote-reference">
                    <a href="#4">
                      3
                    </a>
                  </sup>
                </p>
                <p>
                  baz
                  <sup class="footnote-reference">
                    <a href="#3">
                      4
                    </a>
                  </sup>
                </p>
                <div class="footnote-definition" id="1">
                  <sup class="footnote-definition-label">
                    1
                  </sup>
                  <p>
                    foo
                  </p>
                </div>
                <div class="footnote-definition" id="2">
                  <sup class="footnote-definition-label">
                    2
                  </sup>
                  <p>
                    bar
                  </p>
                </div>
                <div class="footnote-definition" id="3">
                  <sup class="footnote-definition-label">
                    4
                  </sup>
                  <p>
                    baz
                  </p>
                </div>
                <p>
                  quux
                </p>
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

    def test_gfm(self) -> None:
        html = """
        <blockquote class="markdown-alert-note">
          <p>
            foo
          </p>
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
          <dt>
            foo
          </dt>
          <dd>
            bar
          </dd>
          <dt>
            baz
          </dt>
          <dd>
            qux
          </dd>
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
        <p>
          <sup>
            foo
          </sup>
        </p>
        """

        markdown = """
        ^foo^
        """

        TestRender.assert_render(html, markdown, Options(superscript=True))

    def test_subscript(self) -> None:
        html = """
        <p>
          <sub>
            foo
          </sub>
        </p>
        """

        markdown = """
        ~foo~
        """

        TestRender.assert_render(html, markdown, Options(subscript=True))

    def test_wikilinks(self) -> None:
        html = """
        <p>
          <a href="foo">
            foo
          </a>
        </p>
        """

        markdown = """
        [[foo]]
        """

        TestRender.assert_render(html, markdown, Options(wikilinks=True))

    def test_math_inline(self) -> None:
        html = r"""
        <p>
          <math display="inline" xmlns="inline">
            <mrow>
              <msubsup>
                <mo>
                  ∫
                </mo>
                <mn>
                  0
                </mn>
                <mn>
                  1
                </mn>
              </msubsup>
              <mi>
                x
              </mi>
              <mspace width="0.167em">
              </mspace>
              <mi>
                d
              </mi>
              <mi>
                x
              </mi>
            </mrow>
          </math>
        </p>
        """

        markdown = r"""
        $\int_0^1 x \, dx$
        """

        TestRender.assert_render(html, markdown, Options(math=TestRender.math_callback))

    def test_math_display(self) -> None:
        html = r"""
        <p>
          <math display="inline" xmlns="display">
            <mrow>
              <msubsup>
                <mo>
                  ∫
                </mo>
                <mn>
                  0
                </mn>
                <mn>
                  1
                </mn>
              </msubsup>
              <mi>
                x
              </mi>
              <mspace width="0.167em">
              </mspace>
              <mi>
                d
              </mi>
              <mi>
                x
              </mi>
            </mrow>
          </math>
        </p>
        """

        markdown = r"""
        $$\int_0^1 x \, dx$$
        """

        TestRender.assert_render(html, markdown, Options(math=TestRender.math_callback))

    def test_highlight_anonymous(self) -> None:
        html = """
        <html>
          <body>
            <div class="highlight">
              <pre><span></span>let x
            = 1;
        </pre>
            </div>
          </body>
        </html>
        """

        markdown = """
        ```
        let x
            = 1;
        ```
        """

        TestRender.assert_render(html, markdown, Options(code=TestRender.code_callback))

    def test_highlight_language(self) -> None:
        html = """
        <html>
          <body>
            <div class="highlight">
              <pre><span></span><span class="kd">let</span><span class="w"></span><span class="n">x</span>
        <span class="w"></span><span class="o">=</span><span class="w"></span><span class="mi">1</span><span class="p">;</span>
        </pre>
            </div>
          </body>
        </html>
        """  # noqa: E501

        markdown = """
        ```rust
        let x
            = 1;
        ```
        """

        TestRender.assert_render(html, markdown, Options(code=TestRender.code_callback))
