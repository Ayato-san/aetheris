# Debug combat capture and evidence pipeline

Status: accepted

Aetheris will provide a debug-only combat analysis page backed by one packet-ingestion contract shared by an offline PCAPNG adapter and a later live adapter. The offline adapter must be validated against the supplied captures before live capture is connected to the same parser and TCP reassembly pipeline. Network analysis will be implemented in Rust without Npcap or a third-party packet-analysis library; live Windows capture uses native Winsock, filters connections by the AION 2 process, and requires administrator privileges. This keeps file and live analysis comparable while accepting the maintenance cost of a custom parser and the Windows-only, elevated live-capture path.

## Considered options

- Use Npcap or another packet-analysis library for live capture and protocol parsing. Rejected because the project requires no external network-analysis library and no Npcap.
- Implement separate file and live decoding pipelines. Rejected because their results and error behavior could diverge; both adapters instead supply original packet bytes and capture metadata to the same analysis pipeline.

## Consequences

- PCAPNG import retains packet bytes, timestamps, interface/link metadata, source name, and file fingerprint in SQLite at `%LOCALAPPDATA%\Aetheris\debug-captures.sqlite3`; it does not duplicate the complete source file. Corrupt input may produce a partial session, and parsing stops when the next block boundary cannot be trusted.
- Sessions retain raw packets, reconstructed application messages, and zero or more decoded sections until explicit, confirmed deletion. Reanalysis replaces interpretations only after a complete successful run; raw evidence remains unchanged.
- By default, the analyzer fails closed for incomplete or ambiguous streams. Mid-connection starts, TCP gaps or conflicting overlaps, truncated packets, unsupported link/network forms, and fragmented IP traffic remain visible as raw evidence with an error or unsupported status rather than guessed messages.
- The live adapter waits for AION 2 connections, follows a relaunch when exactly one client process is present, and reports ambiguity instead of selecting among multiple matching processes. Interface selection is inferred from the client socket when possible, with manual selection as fallback; permission failures do not trigger automatic elevation.
- Offline packet import remains available independently of the Windows-only live adapter. The debug page is excluded from production builds.
- Live capture is limited to IPv4 in this version. Winsock raw IPv6 reception does not provide the original IPv6 header, so it cannot satisfy the original-packet contract; IPv6 remains supported for offline analysis. See [Microsoft raw-socket receive rules](https://learn.microsoft.com/en-us/windows/win32/winsock/tcp-ip-raw-sockets-2).
- Midstream alignment remains disabled by default. An explicit, persisted offline-only option may assume a message boundary at the first reconstructed captured byte; the page and resulting message rows identify this as an experiment. Both supplied PCAPNG files lack SYN packets, so this opt-in is needed to study their provisional framing without presenting it as established.
- rusqlite supplies SQLite storage, sha2 supplies source fingerprints, and windows-sys supplies native Windows declarations. None of these dependencies performs packet capture analysis, protocol parsing, or TCP reassembly.
