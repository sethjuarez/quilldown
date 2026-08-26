# Endnotes

Markdown footnotes become a single, numbered **Notes** section, with clickable cross-links
between each reference mark and its note.

## Body with references

The transformer architecture[^attn] replaced recurrent models for many sequence tasks.
Later work scaled it dramatically[^scale], and the same attention mechanism[^attn] underlies
most modern large language models.

A second, unrelated claim needs a citation too.[^data]

## More prose

Footnotes can be referenced far from where they are defined, and a note referenced multiple
times (like the attention note above) is still listed **once** in the Notes section.

[^attn]: Vaswani et al., *Attention Is All You Need* (2017).
[^scale]: Kaplan et al., *Scaling Laws for Neural Language Models* (2020).
[^data]: A deduplicated note appears once no matter how many times it is cited.
