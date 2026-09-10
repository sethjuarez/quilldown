use quilldown::ir::{lower, spec_wire::SpecDocument};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct VectorManifest {
    vectors: Vec<VectorEntry>,
}

#[derive(Deserialize)]
struct VectorEntry {
    contract: String,
    operation: String,
    vector: Vector,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    input: Value,
    expected: Option<Value>,
}

#[test]
fn rust_lower_conforms_to_generated_vector_corpus() {
    let manifest_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../python/.typra-generated/vectors.json"
    );
    let raw = std::fs::read_to_string(manifest_path).unwrap_or_else(|error| {
        panic!(
            "generated vector corpus is required at {manifest_path}: {error}; \
             run `cd spec && npm run generate` before cargo test"
        )
    });
    let manifest: VectorManifest = serde_json::from_str(&raw).expect("vectors.json is valid JSON");
    let mut checked = 0usize;

    for entry in manifest.vectors {
        if entry.contract != "Quilldown" || entry.operation != "lower" {
            continue;
        }
        let markdown = entry
            .vector
            .input
            .get("markdown")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{} must include input.markdown", entry.vector.name));
        let expected = entry
            .vector
            .expected
            .as_ref()
            .unwrap_or_else(|| panic!("{} must include expected", entry.vector.name));
        let observed = serde_json::to_value(SpecDocument::from(&lower(markdown)))
            .expect("project observed IR to spec wire JSON");

        assert_eq!(
            &observed, expected,
            "Rust lower output drifted from vector {}",
            entry.vector.name
        );
        checked += 1;
    }

    assert!(checked > 0, "no Quilldown.lower vectors found");
}
