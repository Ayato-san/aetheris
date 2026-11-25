PRAGMA foreign_keys = ON;
PRAGMA application_id = 0x41454954;
PRAGMA user_version = 10;

CREATE TABLE client_version (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    steam_build TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_families (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_categories (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    family_id SMALLINT NOT NULL REFERENCES item_families(id) CHECK(family_id BETWEEN 1 AND 32767),
    code TEXT NOT NULL,
    UNIQUE(family_id,code)
) WITHOUT ROWID;
CREATE TABLE armor_types (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE flags (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE rarities (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    rarity TEXT NOT NULL,
    rarity_source TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE sell_currencies (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE,
    sell_currency_type TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE stats (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE,
    name_en TEXT,
    name_localized TEXT
) WITHOUT ROWID;
CREATE TABLE races (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) WITHOUT ROWID;
INSERT INTO races VALUES (1,'All','All'),(2,'Light','Elyos'),(3,'Dark','Asmodian');
CREATE TABLE sets (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE localization_language (
    id INTEGER PRIMARY KEY CHECK(id=1),
    locale TEXT NOT NULL
);
CREATE TABLE catalogue_icons(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),reference TEXT NOT NULL UNIQUE,
 image BLOB,mime_type TEXT,
 image_state TEXT NOT NULL DEFAULT 'not_researched' CHECK(image_state IN('available','not_found','not_researched','ambiguous')),
 image_sha256 TEXT
) WITHOUT ROWID;

CREATE TABLE items (
    id INTEGER PRIMARY KEY CHECK(id > 0),
    client_version_id SMALLINT NOT NULL REFERENCES client_version(id) CHECK(client_version_id BETWEEN 1 AND 32767),
    category_id SMALLINT NOT NULL REFERENCES item_categories(id) CHECK(category_id BETWEEN 1 AND 32767),
    armor_type_id SMALLINT REFERENCES armor_types(id) CHECK(armor_type_id BETWEEN 1 AND 32767),
    rarity_id SMALLINT NOT NULL REFERENCES rarities(id) CHECK(rarity_id BETWEEN 1 AND 32767),
    sell_currency_id SMALLINT NOT NULL REFERENCES sell_currencies(id) CHECK(sell_currency_id BETWEEN 1 AND 32767),
    race_id SMALLINT REFERENCES races(id) CHECK(race_id BETWEEN 1 AND 32767),
    name_en TEXT,
    name_localized TEXT,
    description_en TEXT,
    description_localized TEXT,
    item_level INTEGER NOT NULL CHECK(item_level >= 0),
    sell_price INTEGER NOT NULL CHECK(sell_price >= 0),
    magic_stone_slots TINYINT CHECK(magic_stone_slots IS NULL OR (typeof(magic_stone_slots)='integer' AND magic_stone_slots BETWEEN 0 AND 255)),
    godstone_slots TINYINT CHECK(godstone_slots IS NULL OR (typeof(godstone_slots)='integer' AND godstone_slots BETWEEN 0 AND 255))
);
CREATE INDEX items_category ON items(category_id);
CREATE INDEX items_rarity ON items(rarity_id,item_level);
CREATE INDEX items_client_version ON items(client_version_id);
CREATE INDEX items_name_localized ON items(name_localized);
CREATE TABLE item_flags (
    item_id INTEGER NOT NULL REFERENCES items(id),
    flag_id SMALLINT NOT NULL REFERENCES flags(id) CHECK(flag_id BETWEEN 1 AND 32767),
    PRIMARY KEY(item_id,flag_id)
) WITHOUT ROWID;
CREATE TABLE item_main_stats (
    item_id INTEGER NOT NULL REFERENCES items(id),
    position INTEGER NOT NULL CHECK(position >= 0),
    stat_id SMALLINT NOT NULL REFERENCES stats(id) CHECK(stat_id BETWEEN 1 AND 32767),
    value INTEGER NOT NULL,
    PRIMARY KEY(item_id,position)
) WITHOUT ROWID;
CREATE TABLE item_sets (
    item_id INTEGER NOT NULL REFERENCES items(id),
    set_id SMALLINT NOT NULL REFERENCES sets(id) CHECK(set_id BETWEEN 1 AND 32767),
    PRIMARY KEY(item_id,set_id)
) WITHOUT ROWID;
