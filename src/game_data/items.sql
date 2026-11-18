PRAGMA foreign_keys = ON;
PRAGMA application_id = 0x41454954;
PRAGMA user_version = 4;

CREATE TABLE client_version (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    steam_build TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_families (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_categories (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    family_id SMALLINT NOT NULL REFERENCES item_families(id) CHECK(family_id BETWEEN 1 AND 65535),
    code TEXT NOT NULL,
    UNIQUE(family_id,code)
) WITHOUT ROWID;
CREATE TABLE armor_types (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE cooldowns (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    value INTEGER NOT NULL UNIQUE CHECK(value >= 0)
) WITHOUT ROWID;
CREATE TABLE flags (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE rarities (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    rarity TEXT NOT NULL,
    rarity_source TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE sell_currencies (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE,
    sell_currency_type TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE stats (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE,
    name_en TEXT,
    name_localized TEXT
) WITHOUT ROWID;
CREATE TABLE races (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) WITHOUT ROWID;
INSERT INTO races VALUES (1,'All','All'),(2,'Light','Elyos'),(3,'Dark','Asmodian');
CREATE TABLE sets (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE item_kinds (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE exchange_types (
    id SMALLINT PRIMARY KEY NOT NULL CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 65535),
    code TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
CREATE TABLE localization_language (
    id INTEGER PRIMARY KEY CHECK(id=1),
    locale TEXT NOT NULL
);
CREATE TABLE items (
    id INTEGER PRIMARY KEY CHECK(id > 0),
    client_version_id SMALLINT NOT NULL REFERENCES client_version(id) CHECK(client_version_id BETWEEN 1 AND 65535),
    category_id SMALLINT NOT NULL REFERENCES item_categories(id) CHECK(category_id BETWEEN 1 AND 65535),
    armor_type_id SMALLINT REFERENCES armor_types(id) CHECK(armor_type_id BETWEEN 1 AND 65535),
    cooldown_id SMALLINT REFERENCES cooldowns(id) CHECK(cooldown_id BETWEEN 1 AND 65535),
    rarity_id SMALLINT NOT NULL REFERENCES rarities(id) CHECK(rarity_id BETWEEN 1 AND 65535),
    sell_currency_id SMALLINT NOT NULL REFERENCES sell_currencies(id) CHECK(sell_currency_id BETWEEN 1 AND 65535),
    race_id SMALLINT REFERENCES races(id) CHECK(race_id BETWEEN 1 AND 65535),
    name_en TEXT,
    name_localized TEXT,
    description_en TEXT,
    description_localized TEXT,
    kind_id SMALLINT NOT NULL REFERENCES item_kinds(id) CHECK(kind_id BETWEEN 1 AND 65535),
    item_level INTEGER NOT NULL CHECK(item_level >= 0),
    icon_key TEXT,
    sell_price INTEGER NOT NULL CHECK(sell_price >= 0),
    extensible BOOLEAN CHECK(extensible IN (0,1)),
    skin_extraction_rate INTEGER CHECK(skin_extraction_rate >= 0),
    max_stack_count INTEGER NOT NULL CHECK(max_stack_count >= 0),
    exchange_type_id SMALLINT NOT NULL REFERENCES exchange_types(id) CHECK(exchange_type_id BETWEEN 1 AND 65535),
    use_effect TEXT,
    magic_stone_slots TINYINT CHECK(magic_stone_slots IS NULL OR (typeof(magic_stone_slots)='integer' AND magic_stone_slots BETWEEN 0 AND 255)),
    godstone_slots TINYINT CHECK(godstone_slots IS NULL OR (typeof(godstone_slots)='integer' AND godstone_slots BETWEEN 0 AND 255))
);
CREATE INDEX items_category ON items(category_id);
CREATE INDEX items_rarity ON items(rarity_id,item_level);
CREATE INDEX items_client_version ON items(client_version_id);
CREATE INDEX items_name_localized ON items(name_localized);
CREATE TABLE item_flags (
    item_id INTEGER NOT NULL REFERENCES items(id),
    flag_id SMALLINT NOT NULL REFERENCES flags(id) CHECK(flag_id BETWEEN 1 AND 65535),
    PRIMARY KEY(item_id,flag_id)
) WITHOUT ROWID;
CREATE TABLE item_class_permissions (
    item_id INTEGER NOT NULL REFERENCES items(id),
    class_index INTEGER NOT NULL CHECK(class_index >= 0),
    PRIMARY KEY(item_id,class_index)
) WITHOUT ROWID;
CREATE TABLE item_main_stats (
    item_id INTEGER NOT NULL REFERENCES items(id),
    position INTEGER NOT NULL CHECK(position >= 0),
    stat_id SMALLINT NOT NULL REFERENCES stats(id) CHECK(stat_id BETWEEN 1 AND 65535),
    value INTEGER NOT NULL,
    PRIMARY KEY(item_id,position)
) WITHOUT ROWID;
CREATE TABLE item_extraction_skins (
    item_id INTEGER NOT NULL REFERENCES items(id),
    skin_key TEXT NOT NULL,
    PRIMARY KEY(item_id,skin_key)
) WITHOUT ROWID;
CREATE TABLE item_sets (
    item_id INTEGER NOT NULL REFERENCES items(id),
    set_id SMALLINT NOT NULL REFERENCES sets(id) CHECK(set_id BETWEEN 1 AND 65535),
    PRIMARY KEY(item_id,set_id)
) WITHOUT ROWID;
