CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL, source TEXT NOT NULL,
                started_at INTEGER NOT NULL, status TEXT NOT NULL,
                fingerprint TEXT NOT NULL DEFAULT '', error TEXT,
                decoder_version TEXT NOT NULL DEFAULT '', experimental_midstream INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS packets (
                session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                number INTEGER NOT NULL, section_id INTEGER NOT NULL, interface_id INTEGER NOT NULL,
                link_type INTEGER NOT NULL, timestamp_ns TEXT, timestamp_ticks TEXT,
                timestamp_resolution INTEGER, timestamp_offset INTEGER, local_address TEXT, captured_len INTEGER NOT NULL,
                original_len INTEGER NOT NULL, bytes BLOB NOT NULL,
                PRIMARY KEY(session_id, number)
             );
             CREATE TABLE IF NOT EXISTS analysis_rows (
                id INTEGER PRIMARY KEY, session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                ordinal INTEGER NOT NULL, kind TEXT NOT NULL, timestamp_ns TEXT,
                source TEXT NOT NULL, destination TEXT NOT NULL, direction TEXT NOT NULL,
                protocol TEXT NOT NULL, opcode TEXT NOT NULL, entity TEXT NOT NULL,
                status TEXT NOT NULL, summary TEXT NOT NULL, bytes BLOB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS analysis_rows_session ON analysis_rows(session_id, ordinal);
             CREATE TABLE IF NOT EXISTS row_packets (
                row_id INTEGER NOT NULL REFERENCES analysis_rows(id) ON DELETE CASCADE,
                session_id INTEGER NOT NULL, packet_number INTEGER NOT NULL,
                FOREIGN KEY(session_id, packet_number) REFERENCES packets(session_id, number) ON DELETE CASCADE,
                PRIMARY KEY(row_id, packet_number)
             );
             CREATE TABLE IF NOT EXISTS decoded_sections (
                id INTEGER PRIMARY KEY, row_id INTEGER NOT NULL REFERENCES analysis_rows(id) ON DELETE CASCADE,
                kind TEXT NOT NULL, value TEXT NOT NULL, confidence TEXT NOT NULL,
                byte_offset INTEGER NOT NULL, bytes BLOB NOT NULL
             );
             PRAGMA user_version=1;
