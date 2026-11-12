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
