-- Shared evidence for projected catalogue families.
CREATE TABLE catalogue_codes(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
 code TEXT NOT NULL UNIQUE,name_en TEXT,name_localized TEXT
) WITHOUT ROWID;
CREATE TABLE catalogue_entity_sources(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL CHECK(source_row>=0),
 PRIMARY KEY(family,source_id,source_name,source_row)
) WITHOUT ROWID;
CREATE TABLE catalogue_references(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 target_table TEXT NOT NULL,target_field TEXT NOT NULL,target_reference TEXT NOT NULL,target_row INTEGER,
 resolution_state TEXT NOT NULL CHECK(resolution_state IN('resolved','group','missing','ambiguous','not_researched')),
 interpretation_state TEXT NOT NULL CHECK(interpretation_state IN('verified','candidate')),
 CHECK((resolution_state='resolved')=(target_row IS NOT NULL)),
 PRIMARY KEY(family,source_id,source_name,source_row,source_path,target_table,target_field)
) WITHOUT ROWID;
CREATE INDEX catalogue_references_target ON catalogue_references(target_table,target_field,target_reference);
CREATE TABLE catalogue_conditions(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 kind_id SMALLINT REFERENCES catalogue_codes(id),value_json TEXT NOT NULL CHECK(json_valid(value_json)),
 interpretation_state TEXT NOT NULL CHECK(interpretation_state IN('verified','uninterpreted')),
 PRIMARY KEY(family,source_id,source_name,source_row,source_path)
) WITHOUT ROWID;
