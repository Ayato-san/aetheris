# Preserve the provenance of compressed AION 2 messages

Status: accepted

A compressed AION 2 envelope contains complete application messages whose
bytes differ from the captured compressed bytes. Aetheris represents each
inner message separately, retains an explicit relationship to its inspectable
enclosing envelope and source packets, and distinguishes decompressed offsets
from captured-byte offsets. Linking inner messages only to packets would lose
the transformation that produced their bytes; retaining the envelope relation
adds storage and navigation complexity but keeps interpretations auditable
when framing or decoding changes.

## Implementation — 3 November 2026

Implemented in decoder `aion2-experimental-4`. [Analysis rows](../../src/combat_debug/analysis.rs)
record either an offset in a reconstructed directional TCP stream or an offset
in a specific enclosing envelope’s decompressed output. Neither is a packet-byte
offset. [LZ4 expansion](../../src/combat_debug/compression.rs) retains nested
envelopes and undecoded trailing bytes as inspectable rows, with source-packet
links inherited by their children.

[SQLite provenance](../../src/companion/capture_provenance.sql) persists the
envelope relationship within each session. Reanalysis replaces interpretations
transactionally and preserves captured packets; older interpretations have
unknown provenance until reanalysis. The debug detail displays the enclosing
envelope chain and original packets even when table filters hide the envelopes.

Direct and decompressed messages share [S2C interpretation](../../src/combat_debug/interpretation.rs).
The opcode catalogue and mid-connection synchronization are related pipeline
changes, described with their limits in the
[current implementation report](../protocol-decoding.md#état-de-limplémentation).

Validation covers nested and marked bundles, TCP segmentation, malformed or
excessive decompression, retained evidence, migration and transactional
reanalysis. Both supplied captures can be replayed, but neither contains a
recognized LZ4 envelope: compression and its provenance are validated with
synthetic vectors, not yet with a complete real-game compressed session.
Manual Windows live validation remains outstanding.
