-- #82: source IDs remain distinct from associated item IDs.
CREATE TABLE pet_families(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE,
 name_en TEXT,name_localized TEXT,buff_title_en TEXT,buff_title_localized TEXT,buff_description_en TEXT,buff_description_localized TEXT,
 currency_id SMALLINT REFERENCES catalogue_codes(id),currency_count_raw INTEGER,
 soul_type_1_id SMALLINT REFERENCES catalogue_codes(id),soul_type_2_id SMALLINT REFERENCES catalogue_codes(id),soul_count_raw INTEGER
) WITHOUT ROWID;
CREATE TABLE pet_option_groups(id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE pet_stat_groups(id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE pet_family_slots(
 family_id SMALLINT NOT NULL REFERENCES pet_families(id),position INTEGER NOT NULL,open_level INTEGER,option_group_id SMALLINT REFERENCES pet_option_groups(id),
 PRIMARY KEY(family_id,position)
) WITHOUT ROWID;
CREATE TABLE pet_family_lock_costs(
 family_id SMALLINT NOT NULL REFERENCES pet_families(id),position INTEGER NOT NULL,soul_count_raw INTEGER,currency_count_raw INTEGER,
 PRIMARY KEY(family_id,position)
) WITHOUT ROWID;
CREATE TABLE pets(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,family_id SMALLINT REFERENCES pet_families(id),
 item_reference TEXT,item_id INTEGER REFERENCES items(id),soul_item_reference TEXT,soul_item_id INTEGER REFERENCES items(id),
 stat_group_reference TEXT,stat_group_id SMALLINT REFERENCES pet_stat_groups(id),required_souls_raw INTEGER,
 soul_type_id SMALLINT REFERENCES catalogue_codes(id),polymorph_reference INTEGER,resource_key TEXT,
 display_order INTEGER,is_open BOOLEAN CHECK(is_open IN(0,1)),polymorph_planned BOOLEAN CHECK(polymorph_planned IN(0,1))
);
CREATE TABLE pet_family_levels(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),family_id SMALLINT REFERENCES pet_families(id),
 level INTEGER NOT NULL,required_exp_raw INTEGER,gained_exp_raw INTEGER,UNIQUE(family_id,level)
);
CREATE TABLE pet_grade_weights(
 level_id INTEGER NOT NULL REFERENCES pet_family_levels(id),grade_source TEXT NOT NULL,weight_raw INTEGER,
 PRIMARY KEY(level_id,grade_source)
) WITHOUT ROWID;
CREATE TABLE pet_options(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),group_id SMALLINT REFERENCES pet_option_groups(id),
 grade_id SMALLINT REFERENCES catalogue_codes(id),stat_id SMALLINT REFERENCES stats(id),value_min_raw INTEGER,value_max_raw INTEGER,weight_raw INTEGER
);
CREATE INDEX pet_options_group ON pet_options(group_id);
CREATE TABLE pet_stat_levels(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),group_id SMALLINT REFERENCES pet_stat_groups(id),level INTEGER,
 UNIQUE(group_id,level)
);
CREATE TABLE pet_stat_values(
 level_id INTEGER NOT NULL REFERENCES pet_stat_levels(id),position INTEGER NOT NULL,stat_id SMALLINT NOT NULL REFERENCES stats(id),value_raw INTEGER,
 PRIMARY KEY(level_id,position)
) WITHOUT ROWID;
CREATE TABLE pet_levels(level INTEGER PRIMARY KEY,required_souls_raw INTEGER,gained_family_exp_raw INTEGER);
CREATE TABLE pet_collections(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),required_count_raw INTEGER,
 reward_item_reference TEXT,reward_item_id INTEGER REFERENCES items(id),reward_item_count_raw INTEGER,
 reward_pet_reference TEXT,reward_pet_id INTEGER REFERENCES pets(id)
);
CREATE TABLE wing_effect_groups(id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE wing_enchant_groups(id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE) WITHOUT ROWID;
CREATE TABLE wings(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),name_en TEXT,name_localized TEXT,
 grade_id SMALLINT REFERENCES catalogue_codes(id),race_id SMALLINT REFERENCES races(id),costume_key TEXT,
 effect_group_reference TEXT,effect_group_id SMALLINT REFERENCES wing_effect_groups(id),enchant_group_reference TEXT,enchant_group_id SMALLINT REFERENCES wing_enchant_groups(id),
 display_order INTEGER,show BOOLEAN CHECK(show IN(0,1))
);
CREATE TABLE wing_effect_levels(
 group_id SMALLINT NOT NULL REFERENCES wing_effect_groups(id),level INTEGER NOT NULL,PRIMARY KEY(group_id,level)
) WITHOUT ROWID;
CREATE TABLE wing_effect_stats(
 group_id SMALLINT NOT NULL,level INTEGER NOT NULL,position INTEGER NOT NULL,stat_id SMALLINT NOT NULL REFERENCES stats(id),value_raw INTEGER,
 PRIMARY KEY(group_id,level,position),FOREIGN KEY(group_id,level) REFERENCES wing_effect_levels(group_id,level)
) WITHOUT ROWID;
CREATE TABLE wing_enchant_levels(
 group_id SMALLINT NOT NULL REFERENCES wing_enchant_groups(id),level INTEGER NOT NULL,basic_prob_raw INTEGER,add_prob_raw INTEGER,max_prob_raw INTEGER,
 PRIMARY KEY(group_id,level)
) WITHOUT ROWID;
CREATE TABLE wing_enchant_costs(
 group_id SMALLINT NOT NULL,level INTEGER NOT NULL,position INTEGER NOT NULL,item_reference TEXT,item_id INTEGER REFERENCES items(id),count_raw INTEGER,add_prob_count_raw INTEGER,
 PRIMARY KEY(group_id,level,position),FOREIGN KEY(group_id,level) REFERENCES wing_enchant_levels(group_id,level)
) WITHOUT ROWID;
CREATE TABLE skins(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 category_id SMALLINT REFERENCES catalogue_codes(id),grade_id SMALLINT REFERENCES catalogue_codes(id),style_id SMALLINT REFERENCES catalogue_codes(id),
 abnormal_reference INTEGER,abnormal_id INTEGER REFERENCES skill_abnormals(id),release_date_source TEXT,display_order INTEGER,
 show BOOLEAN CHECK(show IN(0,1)),can_decorate BOOLEAN CHECK(can_decorate IN(0,1)),multiple_ownership BOOLEAN CHECK(multiple_ownership IN(0,1))
);
CREATE TABLE skin_variants(
 skin_id INTEGER NOT NULL REFERENCES skins(id),race_id SMALLINT NOT NULL REFERENCES races(id),sex_id SMALLINT NOT NULL CHECK(sex_id IN(1,2)),
 name_en TEXT,name_localized TEXT,icon_id SMALLINT REFERENCES catalogue_icons(id),costume_resource_key TEXT,
 PRIMARY KEY(skin_id,race_id,sex_id)
) WITHOUT ROWID;
CREATE TABLE skin_sets(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_light_en TEXT,name_light_localized TEXT,name_dark_en TEXT,name_dark_localized TEXT,description_en TEXT,description_localized TEXT,
 grade_id SMALLINT REFERENCES catalogue_codes(id),category_id SMALLINT REFERENCES catalogue_codes(id),release_date_source TEXT,display_order INTEGER,
 show BOOLEAN CHECK(show IN(0,1)),appearance_override_reference TEXT,appearance_override_change BOOLEAN CHECK(appearance_override_change IN(0,1))
);
CREATE TABLE skin_set_members(
 set_id INTEGER NOT NULL REFERENCES skin_sets(id),position INTEGER NOT NULL,skin_reference TEXT NOT NULL,skin_id INTEGER REFERENCES skins(id),
 PRIMARY KEY(set_id,position)
) WITHOUT ROWID;
CREATE TABLE appearance_stats(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,role TEXT NOT NULL,position INTEGER NOT NULL,stat_id SMALLINT NOT NULL REFERENCES stats(id),value_raw INTEGER,
 PRIMARY KEY(family,source_id,role,position)
) WITHOUT ROWID;
CREATE TABLE skin_collection_stats(
 category_id SMALLINT NOT NULL REFERENCES catalogue_codes(id),required_points_raw INTEGER NOT NULL,position INTEGER NOT NULL,
 stat_id SMALLINT NOT NULL REFERENCES stats(id),value_raw INTEGER,
 PRIMARY KEY(category_id,required_points_raw,position)
) WITHOUT ROWID;
-- Positional result rows have no game ID and are not tracked as stable entities.
CREATE TABLE skin_combine_results(
 source_row INTEGER NOT NULL,position INTEGER NOT NULL,is_weapon_variant BOOLEAN NOT NULL CHECK(is_weapon_variant IN(0,1)),
 grade_id SMALLINT REFERENCES catalogue_codes(id),result_group_id SMALLINT REFERENCES catalogue_codes(id),weight_raw INTEGER,
 skin_reference INTEGER NOT NULL,skin_id INTEGER REFERENCES skins(id),
 PRIMARY KEY(source_row,position)
) WITHOUT ROWID;
CREATE INDEX skin_combine_target ON skin_combine_results(skin_id);
CREATE TABLE skin_combine_costs(
 grade_id SMALLINT NOT NULL REFERENCES catalogue_codes(id),currency_id SMALLINT REFERENCES catalogue_codes(id),item_reference INTEGER,item_id INTEGER REFERENCES items(id),count_raw INTEGER,
 PRIMARY KEY(grade_id)
) WITHOUT ROWID;
CREATE TABLE skin_gacha_results(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 result_type_id SMALLINT REFERENCES catalogue_codes(id),result_reference TEXT,skin_id INTEGER REFERENCES skins(id),weight_raw INTEGER
);
CREATE TABLE skin_gacha_groups(
 source_row INTEGER PRIMARY KEY,group_reference TEXT,gacha_reference TEXT,gacha_id INTEGER REFERENCES skin_gacha_results(id),weight_raw INTEGER
);
