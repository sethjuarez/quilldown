"""Quilldown Python runtime: the hand-authored `lower`/`emit` seam that
implements the generated `Quilldown` Protocol from the shared spec."""
from __future__ import annotations

from quilldown_spec import Document, RenderStats, RenderOptions, ConvertOptions

from .parser import markdown_to_ir
from .render import compute_stats, render_docx, validate_document

__all__ = ["Runtime", "lower", "emit", "render_docx", "validate_document"]


class Runtime:
    """Concrete implementation of the `Quilldown` seam Protocol."""

    def lower(self, markdown: str, options: "ConvertOptions | None" = None) -> Document:
        return Document.load(markdown_to_ir(markdown))

    async def lower_async(self, markdown: str, options: "ConvertOptions | None" = None) -> Document:
        return self.lower(markdown, options)

    def emit(self, doc: Document, options: "RenderOptions | None" = None) -> RenderStats:
        doc_dict = doc.save() if isinstance(doc, Document) else doc
        return RenderStats.load(compute_stats(doc_dict))

    async def emit_async(self, doc: Document, options: "RenderOptions | None" = None) -> RenderStats:
        return self.emit(doc, options)


_DEFAULT = Runtime()


def lower(markdown: str, options: "ConvertOptions | None" = None) -> Document:
    return _DEFAULT.lower(markdown, options)


def emit(doc: Document, options: "RenderOptions | None" = None) -> RenderStats:
    return _DEFAULT.emit(doc, options)
