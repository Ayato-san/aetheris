-- Existing interpretations have unknown positions until reanalysis.
CREATE UNIQUE INDEX analysis_rows_ordinal ON analysis_rows(session_id, ordinal);
CREATE TABLE row_provenance (
    row_id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL,
    ordinal INTEGER NOT NULL,
    envelope_ordinal INTEGER,
    byte_offset INTEGER NOT NULL CHECK(byte_offset >= 0),
    CHECK(envelope_ordinal IS NULL OR envelope_ordinal < ordinal),
    FOREIGN KEY(session_id, ordinal) REFERENCES analysis_rows(session_id, ordinal) ON DELETE CASCADE,
    FOREIGN KEY(session_id, envelope_ordinal) REFERENCES analysis_rows(session_id, ordinal) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(row_id) REFERENCES analysis_rows(id) ON DELETE CASCADE
);
