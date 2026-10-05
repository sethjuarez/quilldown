"""Quilldown Python runtime: the hand-authored `lower`/`emit` seam that
implements the generated `Quilldown` Protocol from the shared spec."""
from __future__ import annotations

from quilldown_spec import ConvertOptions, Document, RenderOptions, RenderStats

from .parser import markdown_to_ir
from .render import compute_stats, render_docx, validate_document

__all__ = ["Runtime", "emit", "lower", "render", "render_docx", "validate_document"]


class Runtime:
    """Concrete implementation of the `Quilldown` seam Protocol."""

    def lower(self, markdown: str, options: ConvertOptions | None = None) -> Document:
        return Document.load(markdown_to_ir(markdown))

    async def lower_async(self, markdown: str, options: ConvertOptions | None = None) -> Document:
        return self.lower(markdown, options)

    def emit(self, doc: Document, options: RenderOptions | None = None) -> RenderStats:
        doc_dict = doc.save() if isinstance(doc, Document) else doc
        return RenderStats.load(compute_stats(doc_dict))

    async def emit_async(self, doc: Document, options: RenderOptions | None = None) -> RenderStats:
        return self.emit(doc, options)

    def render(self, doc: Document, options: RenderOptions | None = None):
        doc_dict = doc.save() if isinstance(doc, Document) else doc
        opts = options.save() if isinstance(options, RenderOptions) else options
        return render_docx(doc_dict, opts)

    async def render_async(self, doc: Document, options: RenderOptions | None = None):
        return self.render(doc, options)


_DEFAULT = Runtime()


def lower(markdown: str, options: ConvertOptions | None = None) -> Document:
    return _DEFAULT.lower(markdown, options)


def emit(doc: Document, options: RenderOptions | None = None) -> RenderStats:
    return _DEFAULT.emit(doc, options)


def render(doc: Document, options: RenderOptions | None = None):
    return _DEFAULT.render(doc, options)
