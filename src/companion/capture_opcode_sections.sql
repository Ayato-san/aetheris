-- Opcode-name filters and detail reads share the stored section metadata.
CREATE INDEX decoded_sections_row_kind ON decoded_sections(row_id, kind);
