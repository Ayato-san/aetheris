# Combat reference messages

`zikel-messages.json` contains original framed S2C logical messages from the
user-provided Zikel capture described in ADR 9. Each entry is a pair of relative
nanosecond timestamp and hexadecimal bytes. It includes damage messages targeting
38153, the local player identity, and spawn/removal messages for the target and
damage sources, including later reuse of summon identifiers.

The bytes were extracted through Aetheris's TCP reconstruction and LZ4 decoder.
Expected totals come independently from the user's Questlog reconciliation:
4,807,725 damage, 482 direct hits, 347 periodic ticks, 13 skill groups.
The full private PCAPNG remains outside Git; `combat_history_reference` provides
an explicitly ignored end-to-end test that can be run when it is available.

# Hidden cube reference

`src/overlay/cube_decoder.rs` includes the object payload from a2kit's
`game/parse_test.go::TestParseObject`, revision
`b1ffe3d6a1248bc26d906983f96896f99d740195` (MIT, notice in
`LICENSES/a2kit-MIT.txt`). Its independently documented expected identity is
28137, type 1200039, position (−3998, 89851, 22067) cm. Tests also feed it through
the live widget pipeline with every TCP split, reordered segments, retransmission
and LZ4. No full capture or character name is redistributed.
