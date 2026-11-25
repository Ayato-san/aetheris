-- Product supplements and lossless, compressed source evidence for the target.
CREATE TABLE catalogue_manifest(
 id INTEGER PRIMARY KEY CHECK(id=1),build_id SMALLINT NOT NULL REFERENCES client_version(id),locale TEXT NOT NULL,
 steam_manifest_sha256 TEXT NOT NULL,mapping_sha256 TEXT NOT NULL,reader_version TEXT NOT NULL,reader_revision TEXT NOT NULL,
 region TEXT,region_state TEXT NOT NULL CHECK(region_state IN('verified','unknown'))
);
CREATE TABLE catalogue_sources(
 name TEXT PRIMARY KEY, build_id SMALLINT NOT NULL REFERENCES client_version(id),
 mapping_sha256 TEXT NOT NULL,payload_sha256 TEXT NOT NULL,row_struct TEXT NOT NULL,
 compatibility_verified BOOLEAN NOT NULL CHECK(compatibility_verified IN(0,1)),
 state TEXT NOT NULL CHECK(state IN('decoded','retained')),
 row_count INTEGER NOT NULL,decoded_lz4 BLOB NOT NULL
) WITHOUT ROWID;
CREATE TABLE catalogue_coverage(
 subject TEXT PRIMARY KEY,state TEXT NOT NULL CHECK(state IN('verified','uninterpreted','not_found','not_researched','failed','partial')),
 diagnostic TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE catalogue_entries(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,
 content_hash TEXT NOT NULL,localized_hash TEXT NOT NULL,locale TEXT NOT NULL,comparison_profile TEXT NOT NULL,
 present BOOLEAN NOT NULL CHECK(present IN(0,1)),
 observed_build_id SMALLINT NOT NULL REFERENCES client_version(id),
 reference_build_id SMALLINT REFERENCES client_version(id),
 change_type TEXT CHECK(change_type IN('added','modified','deleted')),
 change_build_id SMALLINT REFERENCES client_version(id),
 change_reference_build_id SMALLINT REFERENCES client_version(id),
 comparison_state TEXT NOT NULL CHECK(comparison_state IN('initial','comparable','impossible')),
 data_build_id SMALLINT NOT NULL REFERENCES client_version(id),
 PRIMARY KEY(family,source_id)
) WITHOUT ROWID;
-- Family baselines also exist when the last reliable source contained no rows.
-- A retained source never advances this baseline to the attempted build.
CREATE TABLE catalogue_comparisons(
 family TEXT PRIMARY KEY,comparison_profile TEXT NOT NULL,
 observed_build_id SMALLINT NOT NULL REFERENCES client_version(id),
 reference_build_id SMALLINT REFERENCES client_version(id),
 comparison_state TEXT NOT NULL CHECK(comparison_state IN('initial','comparable','impossible')),
 interval_incomplete BOOLEAN NOT NULL CHECK(interval_incomplete IN(0,1))
) WITHOUT ROWID;
-- Stable local identities for source loot definitions, including tombstones.
-- Several race/level variants can belong to the same source key.
CREATE TABLE loot_tables(
 id INTEGER PRIMARY KEY,
 source_name TEXT NOT NULL,source_key TEXT NOT NULL,
 client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 UNIQUE(source_name,source_key)
);
CREATE TABLE catalogue_entity_icons(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,icon_id SMALLINT NOT NULL REFERENCES catalogue_icons(id),
 PRIMARY KEY(family,source_id)
) WITHOUT ROWID;
CREATE TABLE catalogue_labels(
 enum_type TEXT NOT NULL,code TEXT NOT NULL,string_key TEXT NOT NULL,name_en TEXT,name_localized TEXT,
 PRIMARY KEY(enum_type,code)
) WITHOUT ROWID;
CREATE TABLE ui_sprites(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
 asset_path TEXT NOT NULL UNIQUE,
 icon_id SMALLINT NOT NULL REFERENCES catalogue_icons(id) CHECK(icon_id BETWEEN 1 AND 32767),
 width SMALLINT NOT NULL CHECK(width BETWEEN 1 AND 4096),
 height SMALLINT NOT NULL CHECK(height BETWEEN 1 AND 4096)
) WITHOUT ROWID;
CREATE TABLE ui_sprite_aliases(
 table_name TEXT NOT NULL,row_key TEXT NOT NULL,
 sprite_id SMALLINT NOT NULL REFERENCES ui_sprites(id) CHECK(sprite_id BETWEEN 1 AND 32767),
 PRIMARY KEY(table_name,row_key)
) WITHOUT ROWID;
CREATE TABLE catalogue_texts(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,field TEXT NOT NULL,string_key TEXT,
 english_state TEXT NOT NULL CHECK(english_state IN('available','missing','no_reference')),
 localized_state TEXT NOT NULL CHECK(localized_state IN('available','fallback_en','missing','no_reference')),
 PRIMARY KEY(family,source_id,field)
) WITHOUT ROWID;
CREATE TABLE item_details(
 item_id INTEGER PRIMARY KEY REFERENCES items(id),
 required_level_min INTEGER
);
CREATE TABLE item_category_filters(
 category_id SMALLINT NOT NULL REFERENCES item_categories(id),filter_source TEXT NOT NULL,
 PRIMARY KEY(category_id,filter_source)
) WITHOUT ROWID;
-- True values live in item_flags; absence from both tables means unknown,
-- never an implicitly false permission. No duplicate true membership.
CREATE TABLE item_false_flags(
 item_id INTEGER NOT NULL REFERENCES items(id),flag_id SMALLINT NOT NULL REFERENCES flags(id),
 PRIMARY KEY(item_id,flag_id)
) WITHOUT ROWID;
CREATE TABLE item_set_definitions(
 set_id SMALLINT NOT NULL REFERENCES sets(id),source_row INTEGER NOT NULL,
 name_en TEXT,name_localized TEXT,type_source TEXT,icon_reference TEXT,PRIMARY KEY(set_id,source_row)
) WITHOUT ROWID;
CREATE TABLE item_set_members(
 set_id SMALLINT NOT NULL,definition_row INTEGER NOT NULL,item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),
 PRIMARY KEY(set_id,definition_row,item_reference),
 FOREIGN KEY(set_id,definition_row) REFERENCES item_set_definitions(set_id,source_row)
) WITHOUT ROWID;
CREATE TABLE item_set_bonuses(
 set_id SMALLINT NOT NULL REFERENCES sets(id),source_row INTEGER NOT NULL,
 equipped_count INTEGER,abnormal_id INTEGER,
 PRIMARY KEY(set_id,source_row)
) WITHOUT ROWID;
CREATE TABLE item_progression_links(
 item_id INTEGER NOT NULL REFERENCES items(id),source_name TEXT NOT NULL REFERENCES catalogue_sources(name),
 source_row INTEGER NOT NULL,source_field TEXT NOT NULL,group_reference TEXT NOT NULL,
 level INTEGER,PRIMARY KEY(item_id,source_name,source_row,source_field)
) WITHOUT ROWID;
CREATE TABLE item_obtaining_routes(
 item_id INTEGER NOT NULL REFERENCES items(id),route_type TEXT NOT NULL,source_name TEXT NOT NULL REFERENCES catalogue_sources(name),
 source_row INTEGER NOT NULL,source_path TEXT NOT NULL,target_table TEXT,target_reference TEXT NOT NULL,
 resolved BOOLEAN NOT NULL CHECK(resolved IN(0,1)),
 PRIMARY KEY(item_id,source_row,source_path)
) WITHOUT ROWID;
CREATE INDEX item_obtaining_reverse ON item_obtaining_routes(target_table,target_reference,item_id);
CREATE TABLE item_obtaining_types(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
 code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_obtaining_sources(
 item_id INTEGER NOT NULL REFERENCES items(id),
 type_id SMALLINT NOT NULL REFERENCES item_obtaining_types(id) CHECK(type_id BETWEEN 1 AND 32767),
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 evidence TEXT NOT NULL CHECK(evidence IN('client_route','manual','output')),
 quantity_min INTEGER,quantity_max INTEGER,weight_raw TEXT,
 PRIMARY KEY(item_id,source_name,source_row,source_path)
) WITHOUT ROWID;
CREATE INDEX item_obtaining_sources_type ON item_obtaining_sources(type_id,item_id);
CREATE INDEX item_obtaining_sources_origin ON item_obtaining_sources(source_name,source_row,item_id);
CREATE TABLE item_rewards(
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 container_item_id INTEGER REFERENCES items(id),group_reference TEXT NOT NULL,
 mode TEXT NOT NULL,result_kind TEXT NOT NULL,result_reference TEXT NOT NULL,result_item_id INTEGER REFERENCES items(id),
 quantity_min INTEGER,quantity_max INTEGER,weight_raw TEXT,
 PRIMARY KEY(source_name,source_row,source_path,group_reference)
) WITHOUT ROWID;
CREATE INDEX item_rewards_result ON item_rewards(result_item_id,container_item_id);
CREATE INDEX item_rewards_container ON item_rewards(container_item_id,result_item_id);
CREATE TABLE skill_abnormals(
 id INTEGER PRIMARY KEY,name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 type_source TEXT,display_category_source TEXT,damage_type_source TEXT,
 max_level INTEGER,overlap_count INTEGER,overlap_time_type_source TEXT,replace_type_source TEXT,
 icon_id SMALLINT REFERENCES catalogue_icons(id)
);
CREATE TABLE skill_abnormal_links(
 effect_kind TEXT NOT NULL,effect_id INTEGER NOT NULL,parameter_position INTEGER NOT NULL,
 abnormal_id INTEGER NOT NULL,resolved BOOLEAN NOT NULL CHECK(resolved IN(0,1)),
 interpretation_state TEXT NOT NULL CHECK(interpretation_state='candidate'),
 PRIMARY KEY(effect_kind,effect_id,parameter_position,abnormal_id)
) WITHOUT ROWID;

CREATE TABLE item_relations(
 item_id INTEGER NOT NULL REFERENCES items(id),direction TEXT NOT NULL CHECK(direction IN('source','use')),
 kind TEXT NOT NULL,source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 name_en TEXT NOT NULL,name_localized TEXT,related_item_id INTEGER REFERENCES items(id),
 quantity_min INTEGER,quantity_max INTEGER,weight_raw TEXT,
 probability_state TEXT NOT NULL CHECK(probability_state IN('unknown_scale','not_provided','verified')),
 probability_percent REAL CHECK(probability_percent BETWEEN 0 AND 100),
 PRIMARY KEY(item_id,direction,source_name,source_row,source_path),
 CHECK((probability_state='verified')=(probability_percent IS NOT NULL))
) WITHOUT ROWID;

CREATE TABLE reward_pools(
 id INTEGER PRIMARY KEY,code TEXT NOT NULL UNIQUE,total_weight INTEGER NOT NULL CHECK(total_weight>=0),
 source_row INTEGER NOT NULL UNIQUE,loot_table_id INTEGER REFERENCES loot_tables(id)
);
CREATE TABLE reward_pool_entries(
 pool_id INTEGER NOT NULL REFERENCES reward_pools(id),position INTEGER NOT NULL,item_reference TEXT NOT NULL,
 item_id INTEGER REFERENCES items(id),weight INTEGER NOT NULL CHECK(weight>=0),
 probability_percent REAL CHECK(probability_percent BETWEEN 0 AND 100),quantity_min INTEGER,quantity_max INTEGER,
 PRIMARY KEY(pool_id,position)
) WITHOUT ROWID;
CREATE INDEX reward_pool_items ON reward_pool_entries(item_id,pool_id);
CREATE TABLE dungeon_reward_rolls(
 id INTEGER PRIMARY KEY,dungeon_id INTEGER NOT NULL,name_en TEXT NOT NULL,name_localized TEXT,
 difficulty TEXT NOT NULL,reward_group TEXT NOT NULL,race TEXT NOT NULL,
 pool_id INTEGER REFERENCES reward_pools(id),item_id INTEGER REFERENCES items(id),
 allocation_raw INTEGER NOT NULL CHECK(allocation_raw>=0),guaranteed_draws INTEGER NOT NULL CHECK(guaranteed_draws>=0),
 extra_draw_percent REAL NOT NULL CHECK(extra_draw_percent>=0 AND extra_draw_percent<100),
 quantity_min INTEGER,quantity_max INTEGER,source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 CHECK((pool_id IS NULL)!=(item_id IS NULL)),
 UNIQUE(dungeon_id,reward_group,race,source_name,source_row,source_path)
);
CREATE INDEX dungeon_reward_pool ON dungeon_reward_rolls(pool_id);
CREATE INDEX dungeon_reward_item ON dungeon_reward_rolls(item_id);
