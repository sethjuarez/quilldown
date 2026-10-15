"""Quilldown Python runtime: the hand-authored runtime seam for generated protocols."""
from __future__ import annotations

from quilldown_spec import (
    ConvertOptions,
    Document,
    PowerPointRenderOptions,
    PowerPointRenderResult,
    Presentation,
    RenderOptions,
    RenderStats,
    WordRenderOptions,
    WordRenderResult,
)

from .parser import markdown_to_ir
from .pptx import emit_powerpoint_result, markdown_to_presentation, render_pptx_bytes, validate_presentation
from .render import compute_stats, emit_word_result, render_docx, validate_document

__all__ = [
    "Runtime",
    "lower",
    "emit",
    "render",
    "emit_word",
    "emit_powerpoint",
    "emit_power_point",
    "lower_powerpoint",
    "lower_power_point",
    "render_docx",
    "render_pptx_bytes",
    "validate_document",
    "validate_presentation",
]


class Runtime:
    """Concrete implementation of the generated runtime seam Protocols."""

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

    def emit_word(
        self, doc: Document, options: WordRenderOptions | None = None
    ) -> WordRenderResult:
        typed_doc = doc if isinstance(doc, Document) else Document.load(doc)
        strict = bool(getattr(options, "strict", False)) if options is not None else False
        return WordRenderResult.load(emit_word_result(typed_doc.save(), strict=strict))

    async def emit_word_async(
        self, doc: Document, options: WordRenderOptions | None = None
    ) -> WordRenderResult:
        return self.emit_word(doc, options)

    def emit_powerpoint(
        self, presentation: Presentation, options: PowerPointRenderOptions | None = None
    ) -> PowerPointRenderResult:
        if not isinstance(presentation, Presentation):
            validate_presentation(presentation)
        typed = presentation if isinstance(presentation, Presentation) else Presentation.load(presentation)
        strict = bool(getattr(options, "strict", False)) if options is not None else False
        return PowerPointRenderResult.load(emit_powerpoint_result(typed.save(), strict=strict))

    async def emit_powerpoint_async(
        self, presentation: Presentation, options: PowerPointRenderOptions | None = None
    ) -> PowerPointRenderResult:
        return self.emit_powerpoint(presentation, options)

    def emit_power_point(
        self, presentation: Presentation, options: PowerPointRenderOptions | None = None
    ) -> PowerPointRenderResult:
        return self.emit_powerpoint(presentation, options)

    async def emit_power_point_async(
        self, presentation: Presentation, options: PowerPointRenderOptions | None = None
    ) -> PowerPointRenderResult:
        return self.emit_powerpoint(presentation, options)

    def lower_powerpoint(self, markdown: str, options: ConvertOptions | None = None) -> Presentation:
        return Presentation.load(markdown_to_presentation(markdown))

    async def lower_powerpoint_async(
        self, markdown: str, options: ConvertOptions | None = None
    ) -> Presentation:
        return self.lower_powerpoint(markdown, options)

    def lower_power_point(self, markdown: str, options: ConvertOptions | None = None) -> Presentation:
        return self.lower_powerpoint(markdown, options)

    async def lower_power_point_async(
        self, markdown: str, options: ConvertOptions | None = None
    ) -> Presentation:
        return self.lower_powerpoint(markdown, options)


_DEFAULT = Runtime()


def lower(markdown: str, options: ConvertOptions | None = None) -> Document:
    return _DEFAULT.lower(markdown, options)


def emit(doc: Document, options: RenderOptions | None = None) -> RenderStats:
    return _DEFAULT.emit(doc, options)


def render(doc: Document, options: RenderOptions | None = None):
    return _DEFAULT.render(doc, options)


def emit_word(doc: Document, options: WordRenderOptions | None = None) -> WordRenderResult:
    return _DEFAULT.emit_word(doc, options)


def emit_powerpoint(
    presentation: Presentation, options: PowerPointRenderOptions | None = None
) -> PowerPointRenderResult:
    return _DEFAULT.emit_powerpoint(presentation, options)


def emit_power_point(
    presentation: Presentation, options: PowerPointRenderOptions | None = None
) -> PowerPointRenderResult:
    return _DEFAULT.emit_power_point(presentation, options)


def lower_powerpoint(markdown: str, options: ConvertOptions | None = None) -> Presentation:
    return _DEFAULT.lower_powerpoint(markdown, options)


def lower_power_point(markdown: str, options: ConvertOptions | None = None) -> Presentation:
    return _DEFAULT.lower_power_point(markdown, options)
