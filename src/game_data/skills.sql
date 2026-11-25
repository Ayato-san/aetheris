-- Reference IDs are bounded signed 16-bit positive values; SQLite stores small integers variably.
CREATE TABLE skill_types(id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE skill_damage_types(id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE skill_effect_kinds(id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE skill_weapon_types(id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE skill_classes(id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE skills(
 id INTEGER PRIMARY KEY,
 client_version_id SMALLINT NOT NULL REFERENCES client_version(id) CHECK(client_version_id BETWEEN 1 AND 32767),
 name_en TEXT,name_localized TEXT,
 icon_id SMALLINT REFERENCES catalogue_icons(id),
 type_id SMALLINT REFERENCES skill_types(id),damage_type_id SMALLINT REFERENCES skill_damage_types(id),
 description_en TEXT,description_localized TEXT,
 mp_cost INTEGER,cooldown REAL,range REAL,
 uses_weapon_range BOOLEAN CHECK(uses_weapon_range IN(0,1)),
 max_level INTEGER NOT NULL CHECK(max_level BETWEEN 1 AND 255),
 stagger_damage REAL
);
CREATE TABLE skill_levels(
 skill_id INTEGER NOT NULL REFERENCES skills(id),level INTEGER NOT NULL CHECK(level>0),
 mp_cost INTEGER,cooldown REAL,
 skill_points INTEGER,stigma_points INTEGER,
 PRIMARY KEY(skill_id,level)
) WITHOUT ROWID;
CREATE TABLE skill_weapons(
 skill_id INTEGER NOT NULL REFERENCES skills(id),weapon_id SMALLINT NOT NULL REFERENCES skill_weapon_types(id),
 PRIMARY KEY(skill_id,weapon_id)
) WITHOUT ROWID;
CREATE TABLE skill_cast_times(
 skill_id INTEGER NOT NULL REFERENCES skills(id),level INTEGER NOT NULL CHECK(level>0),stage INTEGER NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('casting','charge_minimum','charge_limit','charge_stage')),
 duration INTEGER NOT NULL,variant_skill_id INTEGER REFERENCES skills(id),
 PRIMARY KEY(skill_id,level,kind,stage)
) WITHOUT ROWID;
CREATE TABLE skill_specializations(
 skill_id INTEGER NOT NULL REFERENCES skills(id),specialization_id INTEGER NOT NULL REFERENCES skills(id),
 required_level INTEGER,description_en TEXT,description_localized TEXT,
 PRIMARY KEY(skill_id,specialization_id)
) WITHOUT ROWID;
-- SE and ABE share a header and ordered parameter storage. Level rows keep the
-- source group's shared definition, avoiding copies for every referencing skill.
CREATE TABLE skill_effects(
 kind_id SMALLINT NOT NULL REFERENCES skill_effect_kinds(id),
 id INTEGER NOT NULL,level_group TEXT,level INTEGER,effect_type TEXT,
 abnormal_id INTEGER,max_level INTEGER,aggro_absolute INTEGER,aggro_ratio INTEGER,target_count_max INTEGER,
 PRIMARY KEY(kind_id,id)
) WITHOUT ROWID;
CREATE UNIQUE INDEX skill_effect_level_lookup ON skill_effects(kind_id,level_group,level) WHERE level IS NOT NULL;
CREATE TABLE skill_effect_values(
 kind_id SMALLINT NOT NULL REFERENCES skill_effect_kinds(id),effect_id INTEGER NOT NULL,position INTEGER NOT NULL CHECK(position>=0),value TEXT NOT NULL,
 PRIMARY KEY(kind_id,effect_id,position),
 FOREIGN KEY(kind_id,effect_id) REFERENCES skill_effects(kind_id,id)
) WITHOUT ROWID;
CREATE TABLE skill_effect_links(
 skill_id INTEGER NOT NULL REFERENCES skills(id),position INTEGER NOT NULL,
 kind_id SMALLINT NOT NULL REFERENCES skill_effect_kinds(id),effect_id INTEGER NOT NULL,filter_id INTEGER,
 PRIMARY KEY(skill_id,position,effect_id),
 FOREIGN KEY(kind_id,effect_id) REFERENCES skill_effects(kind_id,id)
) WITHOUT ROWID;
-- One condition can have several named parameters. No serialized source rows.
CREATE TABLE skill_conditions(
 skill_id INTEGER REFERENCES skills(id),effect_id INTEGER,
 kind TEXT NOT NULL CHECK(kind IN('activation','effect','target')),
 condition_id INTEGER NOT NULL,type TEXT NOT NULL,target TEXT,
 expression BOOLEAN CHECK(expression IN(0,1)),
 parameter TEXT NOT NULL,position INTEGER NOT NULL,value TEXT,
 description_en TEXT,description_localized TEXT,
 CHECK(skill_id IS NOT NULL OR effect_id IS NOT NULL),
 UNIQUE(skill_id,effect_id,kind,condition_id,parameter,position)
);
CREATE INDEX skill_conditions_owner ON skill_conditions(skill_id,effect_id);
CREATE INDEX skill_conditions_effect ON skill_conditions(effect_id) WHERE effect_id IS NOT NULL;
CREATE TABLE skill_class_permissions(skill_id INTEGER NOT NULL REFERENCES skills(id),class_id SMALLINT NOT NULL REFERENCES skill_classes(id),is_listed BOOLEAN NOT NULL CHECK(is_listed IN(0,1)),PRIMARY KEY(skill_id,class_id)) WITHOUT ROWID;
