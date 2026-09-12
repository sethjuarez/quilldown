"""Hand-authored @vector seam adapters, resolved by the generated conformance
suite via the target's `vector-adapter-path` option. Each adapter drives the
real runtime and returns wire-shape values for byte-comparison against the
oracle-derived vectors. No vector is silently skipped."""
from quilldown import Runtime, validate_document, validate_presentation
from quilldown_spec import Document, PowerPointRenderOptions, Presentation, WordRenderOptions

_RUNTIME = Runtime()


def _lower(resolved_input, context):
    doc = _RUNTIME.lower(resolved_input["markdown"])
    return doc.save()


def _emit(resolved_input, context):
    validate_document(resolved_input["doc"])
    doc = Document.load(resolved_input["doc"])
    return _RUNTIME.emit(doc).save()


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


VECTOR_ADAPTERS = {
    "Quilldown.lower": {"invoke": _lower},
    "Quilldown.emit": {"invoke": _emit},
    "WordEmitter.emitWord": {"invoke": _emit_word},
    "PowerPointEmitter.emitPowerPoint": {"invoke": _emit_powerpoint},
}

VECTOR_WAIVERS = {}

VECTOR_DOUBLES = {}
