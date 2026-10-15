"""Hand-authored @vector seam adapters for the generated conformance suite."""
from io import BytesIO

from doc_inspector import rendered_view

from quilldown import Runtime, validate_document, validate_presentation
from quilldown_spec import (
    Document,
    PowerPointRenderOptions,
    Presentation,
    RenderOptions,
    WordRenderOptions,
)

_RUNTIME = Runtime()


def _lower(resolved_input, context):
    doc = _RUNTIME.lower(resolved_input["markdown"])
    return doc.save()


def _emit(resolved_input, context):
    validate_document(resolved_input["doc"])
    doc = Document.load(resolved_input["doc"])
    options = RenderOptions.load(resolved_input["options"]) if "options" in resolved_input else None
    return _RUNTIME.emit(doc, options).save()


def _render(resolved_input, context):
    validate_document(resolved_input["doc"])
    options = RenderOptions.load(resolved_input["options"]) if "options" in resolved_input else None
    buf = BytesIO()
    _RUNTIME.render(Document.load(resolved_input["doc"]), options).save(buf)
    return rendered_view(buf.getvalue())


def _emit_word(resolved_input, context):
    validate_document(resolved_input["doc"])
    doc = Document.load(resolved_input["doc"])
    options = (
        WordRenderOptions.load(resolved_input["options"])
        if "options" in resolved_input
        else None
    )
    return _RUNTIME.emit_word(doc, options).save()


def _emit_powerpoint(resolved_input, context):
    validate_presentation(resolved_input["presentation"])
    presentation = Presentation.load(resolved_input["presentation"])
    options = (
        PowerPointRenderOptions.load(resolved_input["options"])
        if "options" in resolved_input
        else None
    )
    return _RUNTIME.emit_powerpoint(presentation, options).save()


def _lower_powerpoint(resolved_input, context):
    return _RUNTIME.lower_powerpoint(resolved_input["markdown"]).save()


VECTOR_ADAPTERS = {
    "Quilldown.lower": {"invoke": _lower},
    "Quilldown.emit": {"invoke": _emit},
    "Quilldown.render": {"invoke": _render},
    "WordEmitter.emitWord": {"invoke": _emit_word},
    "PowerPointEmitter.lowerPowerPoint": {"invoke": _lower_powerpoint},
    "PowerPointEmitter.emitPowerPoint": {"invoke": _emit_powerpoint},
}

VECTOR_WAIVERS = {}

VECTOR_DOUBLES = {}
