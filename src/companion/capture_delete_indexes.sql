-- Full captures have tens of thousands of rows. Index every cascading child
-- lookup so removing the source capture does not require repeated table scans.
CREATE INDEX row_packets_capture_packet ON row_packets(session_id, packet_number);
CREATE INDEX decoded_sections_row ON decoded_sections(row_id);
CREATE INDEX row_provenance_ordinal ON row_provenance(session_id, ordinal);
CREATE INDEX row_provenance_envelope ON row_provenance(session_id, envelope_ordinal);
