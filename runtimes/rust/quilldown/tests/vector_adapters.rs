use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use quilldown::typra_spec::{
    Document, LoadContext, PowerPointRenderOptions, Presentation, SaveContext, WordRenderOptions,
};
use serde_json::Value;

type AsyncInvoke =
    fn(&Value, &Context) -> Pin<Box<dyn Future<Output = Result<Value, VectorError>> + Send>>;

pub struct Context {
    pub contract: String,
    pub operation: String,
    pub vector: Value,
    pub provider: Option<String>,
    pub target_api: Option<String>,
    pub doubles: Value,
    pub base_dir: String,
}

pub struct Adapter {
    pub invoke: Invoke,
    pub normalize: Option<fn(&Value, &Context) -> Value>,
}

pub enum Invoke {
    Sync(fn(&Value, &Context) -> Result<Value, VectorError>),
    Async(AsyncInvoke),
}

pub struct VectorError {
    pub message: String,
    pub payload: Option<Value>,
}

impl VectorError {
    fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            payload: None,
        }
    }
}

pub fn adapters() -> HashMap<&'static str, Adapter> {
    HashMap::from([
        (
            "Quilldown.lower",
            Adapter {
                invoke: Invoke::Sync(lower_adapter),
                normalize: None,
            },
        ),
        (
            "Quilldown.emit",
            Adapter {
                invoke: Invoke::Sync(emit_adapter),
                normalize: None,
            },
        ),
        (
            "WordEmitter.emitWord",
            Adapter {
                invoke: Invoke::Sync(emit_word_adapter),
                normalize: None,
            },
        ),
        (
            "PowerPointEmitter.lowerPowerPoint",
            Adapter {
                invoke: Invoke::Sync(lower_powerpoint_adapter),
                normalize: None,
            },
        ),
        (
            "PowerPointEmitter.emitPowerPoint",
            Adapter {
                invoke: Invoke::Sync(emit_powerpoint_adapter),
                normalize: None,
            },
        ),
    ])
}

pub fn waivers() -> HashMap<&'static str, &'static str> {
    HashMap::from([(
        "Quilldown.render",
        "Rust render DOCX structural vectors are exercised by the Python harness with the shared doc_inspector; the Rust generated vector harness has no native rendered-view inspector.",
    )])
}

pub fn doubles() -> Value {
    Value::Null
}

fn lower_adapter(input: &Value, _ctx: &Context) -> Result<Value, VectorError> {
    let markdown = input
        .get("markdown")
        .and_then(Value::as_str)
        .ok_or_else(|| VectorError::message("input.markdown is required"))?;
    quilldown::typra_runtime::lower(markdown)
        .map(|doc| doc.to_value(&SaveContext::default()))
        .map_err(vector_error)
}

fn emit_adapter(input: &Value, _ctx: &Context) -> Result<Value, VectorError> {
    let doc = load_doc(input)?;
    quilldown::typra_runtime::emit(&doc, &None)
        .map(|stats| stats.to_value(&SaveContext::default()))
        .map_err(vector_error)
}

fn emit_word_adapter(input: &Value, _ctx: &Context) -> Result<Value, VectorError> {
    let doc = load_doc(input)?;
    let options = match input.get("options") {
        Some(value) => Some(
            WordRenderOptions::try_load_from_value(value, &LoadContext::default())
                .map_err(vector_error)?,
        ),
        None => None,
    };
    quilldown::typra_runtime::emit_word(&doc, &options)
        .map(|result| result.to_value(&SaveContext::default()))
        .map_err(vector_error)
}

fn emit_powerpoint_adapter(input: &Value, _ctx: &Context) -> Result<Value, VectorError> {
    let presentation = load_presentation(input)?;
    let options = match input.get("options") {
        Some(value) => Some(
            PowerPointRenderOptions::try_load_from_value(value, &LoadContext::default())
                .map_err(vector_error)?,
        ),
        None => None,
    };
    quilldown::typra_runtime::emit_powerpoint(&presentation, &options)
        .map(|result| result.to_value(&SaveContext::default()))
        .map_err(vector_error)
}

fn lower_powerpoint_adapter(input: &Value, _ctx: &Context) -> Result<Value, VectorError> {
    let markdown = input
        .get("markdown")
        .and_then(Value::as_str)
        .ok_or_else(|| VectorError::message("input.markdown is required"))?;
    quilldown::typra_runtime::lower_powerpoint(markdown)
        .map(|presentation| presentation.to_value(&SaveContext::default()))
        .map_err(vector_error)
}

fn load_doc(input: &Value) -> Result<Document, VectorError> {
    let value = input
        .get("doc")
        .ok_or_else(|| VectorError::message("input.doc is required"))?;
    quilldown::typra_runtime::load_document_from_value(value).map_err(vector_error)
}

fn load_presentation(input: &Value) -> Result<Presentation, VectorError> {
    let value = input
        .get("presentation")
        .ok_or_else(|| VectorError::message("input.presentation is required"))?;
    quilldown::typra_runtime::load_presentation_from_value(value).map_err(vector_error)
}

fn vector_error(error: impl std::fmt::Display) -> VectorError {
    VectorError {
        message: error.to_string(),
        payload: Some(serde_json::json!({ "message": error.to_string() })),
    }
}
