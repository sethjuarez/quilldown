# Captions and cross-references

Enable captions with the `--captions` flag (or `ConvertOptions::captions`). A paragraph that
starts with `Figure:` or `Table:` becomes an auto-numbered, `Caption`-styled paragraph backed by
a native Word `SEQ` field, so figures and tables number independently and renumber on edit.

Ending a caption with `{#label}` publishes a bookmark. An in-document link to that label renders
a live `REF` cross-reference that Word resolves to the caption's number when the document opens.

## Figures

```text
request --> gateway --> service
```

Figure: End-to-end request flow through the gateway. {#flow}

```text
cache <-- worker <-- queue
```

Figure: The asynchronous worker pipeline. {#pipeline}

As shown in [the request flow](#flow), traffic enters through the gateway; the
[worker pipeline](#pipeline) drains the queue in the background.

## Tables

Table: Supported output targets and their fidelity. {#targets}

| Target | Fidelity |
|--------|----------|
| DOCX   | Native   |
| PDF    | Via Word |

Cross-references count independently: [see the targets table](#targets) is Table 1 even though
two figures precede it. Forward references — like this pointer to [the summary](#summary) — also
resolve, because Word updates every field when the document opens.

Table: Round-trip summary. {#summary}

| Stage  | Status |
|--------|--------|
| Parse  | OK     |
| Render | OK     |
