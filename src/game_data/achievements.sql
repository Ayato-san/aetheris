-- Shared children retain their exact origin and source encoding.
CREATE TABLE catalogue_text_values(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,field TEXT NOT NULL,text_en TEXT,text_localized TEXT,
 PRIMARY KEY(family,source_id,field),
 FOREIGN KEY(family,source_id,field) REFERENCES catalogue_texts(family,source_id,field)
) WITHOUT ROWID;
CREATE TABLE catalogue_item_links(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,role TEXT NOT NULL,position INTEGER NOT NULL,
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,source_path TEXT NOT NULL,
 item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),quantity_raw INTEGER,enchantment INTEGER,threshold_raw INTEGER,
 PRIMARY KEY(family,source_id,source_name,source_row,source_path)
) WITHOUT ROWID;
CREATE INDEX catalogue_items_reverse ON catalogue_item_links(item_id,role,family,source_id);
CREATE TABLE catalogue_periods(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,role TEXT NOT NULL,start_ticks INTEGER,end_ticks INTEGER,
 PRIMARY KEY(family,source_id,role)
) WITHOUT ROWID;
CREATE TABLE catalogue_costs(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,source_path TEXT NOT NULL,position INTEGER,
 currency_id SMALLINT REFERENCES catalogue_codes(id),amount_raw INTEGER,
 PRIMARY KEY(family,source_id,source_path)
) WITHOUT ROWID;
CREATE TABLE achievement_group_sets(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE,
 name_en TEXT,name_localized TEXT,race_id SMALLINT REFERENCES races(id)
) WITHOUT ROWID;
CREATE TABLE achievement_groups(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 category_id SMALLINT REFERENCES catalogue_codes(id),type_id SMALLINT REFERENCES catalogue_codes(id),
 collection_major_type_id SMALLINT REFERENCES catalogue_codes(id),race_id SMALLINT REFERENCES races(id),
 unlock_reference INTEGER,unlock_achievement_id INTEGER REFERENCES achievements(id),
 account_shared BOOLEAN CHECK(account_shared IN(0,1))
);
CREATE TABLE achievement_group_members(
 set_id SMALLINT NOT NULL REFERENCES achievement_group_sets(id),position INTEGER NOT NULL,group_reference TEXT NOT NULL,
 group_id INTEGER REFERENCES achievement_groups(id),PRIMARY KEY(set_id,position)
) WITHOUT ROWID;
CREATE TABLE achievements(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 group_reference TEXT,group_id INTEGER REFERENCES achievement_groups(id),subgroup_reference TEXT,
 objective_type_id SMALLINT REFERENCES catalogue_codes(id),objective_values_json TEXT NOT NULL CHECK(json_valid(objective_values_json)),
 grade_id SMALLINT REFERENCES catalogue_codes(id),reset_cycle_id SMALLINT REFERENCES catalogue_codes(id),
 unlock_reference INTEGER,unlock_achievement_id INTEGER REFERENCES achievements(id),
 collection_content_type_id SMALLINT REFERENCES catalogue_codes(id),collection_subtype_id SMALLINT REFERENCES catalogue_codes(id),
 collection_minor_type_id SMALLINT REFERENCES catalogue_codes(id),display_order INTEGER
);
CREATE INDEX achievements_group ON achievements(group_id);
CREATE TABLE achievement_steps(
 id INTEGER PRIMARY KEY,achievement_reference INTEGER NOT NULL,achievement_id INTEGER REFERENCES achievements(id),
 step_number INTEGER NOT NULL,threshold_raw INTEGER,text_reference TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 UNIQUE(achievement_id,step_number)
);
CREATE TABLE achievement_rewards(
 id INTEGER PRIMARY KEY,achievement_reference INTEGER NOT NULL,achievement_id INTEGER REFERENCES achievements(id),
 step_number INTEGER,step_id INTEGER REFERENCES achievement_steps(id),points_raw INTEGER
);
CREATE TABLE title_categories(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE,
 name_en TEXT,name_localized TEXT,display_order INTEGER
) WITHOUT ROWID;
CREATE TABLE titles(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 category_reference TEXT,category_id SMALLINT REFERENCES title_categories(id),grade_id SMALLINT REFERENCES catalogue_codes(id),
 race_id SMALLINT REFERENCES races(id),apply_target_id SMALLINT REFERENCES catalogue_codes(id),equip_category_id SMALLINT REFERENCES catalogue_codes(id),
 display_order INTEGER,show BOOLEAN CHECK(show IN(0,1)),show_acquisition BOOLEAN CHECK(show_acquisition IN(0,1))
);
CREATE TABLE title_stats(
 title_id INTEGER NOT NULL REFERENCES titles(id),role TEXT NOT NULL CHECK(role IN('possession','equipped')),position INTEGER NOT NULL,
 stat_id SMALLINT NOT NULL REFERENCES stats(id),value_raw INTEGER,
 PRIMARY KEY(title_id,role,position)
) WITHOUT ROWID;
CREATE TABLE season_schedule_groups(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id)
);
CREATE TABLE seasons(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 group_reference TEXT,group_id INTEGER REFERENCES season_schedule_groups(id),season_number INTEGER
);
CREATE TABLE season_contents(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 type_id SMALLINT REFERENCES catalogue_codes(id),season_number INTEGER,schedule_group_reference TEXT,
 schedule_group_id INTEGER REFERENCES season_schedule_groups(id),pickup_reference TEXT,pickup_source_id INTEGER
);
CREATE TABLE season_content_schedules(
 content_id INTEGER NOT NULL REFERENCES season_contents(id),season_id INTEGER NOT NULL REFERENCES seasons(id),
 PRIMARY KEY(content_id,season_id)
) WITHOUT ROWID;
CREATE TABLE event_missions(
 id INTEGER PRIMARY KEY,source_key TEXT,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),name_en TEXT,name_localized TEXT,
 season_reference INTEGER,season_id INTEGER REFERENCES seasons(id),type_id SMALLINT REFERENCES catalogue_codes(id),tab_order INTEGER
);
CREATE TABLE mission_group_sets(
 family TEXT NOT NULL,source_id INTEGER NOT NULL,race_id SMALLINT NOT NULL REFERENCES races(id),position INTEGER NOT NULL,
 source_reference TEXT NOT NULL,group_set_id SMALLINT REFERENCES achievement_group_sets(id),achievement_id INTEGER REFERENCES achievements(id),
 PRIMARY KEY(family,source_id,race_id,position)
) WITHOUT ROWID;
CREATE TABLE battle_pass_groups(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,race_id SMALLINT REFERENCES races(id),season_reference INTEGER,season_id INTEGER REFERENCES seasons(id),
 contents_type_id SMALLINT REFERENCES catalogue_codes(id),buy_limit_raw INTEGER,buy_limit_target_id SMALLINT REFERENCES catalogue_codes(id),
 exp_limit_raw INTEGER,exp_limit_type_id SMALLINT REFERENCES catalogue_codes(id),exp_purchase_item_reference TEXT,
 exp_purchase_item_id INTEGER REFERENCES items(id),tab_order INTEGER
);
CREATE TABLE battle_passes(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,group_reference TEXT,group_id INTEGER REFERENCES battle_pass_groups(id),
 experience_achievement_reference INTEGER,experience_achievement_id INTEGER REFERENCES achievements(id),
 premium_source BOOLEAN CHECK(premium_source IN(0,1)),required_level INTEGER,
 unlock_quest_reference TEXT,unlock_quest_id INTEGER REFERENCES quests(id),abnormal_reference INTEGER,abnormal_id INTEGER REFERENCES skill_abnormals(id)
);
CREATE TABLE supply_request_groups(
 id INTEGER PRIMARY KEY,group_reference TEXT NOT NULL,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 race_id SMALLINT REFERENCES races(id),required_level INTEGER,tab_type_id SMALLINT REFERENCES catalogue_codes(id),show_date BOOLEAN CHECK(show_date IN(0,1))
);
CREATE INDEX supply_groups_reference ON supply_request_groups(group_reference);
CREATE TABLE supply_requests(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),group_reference TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,target_count_raw INTEGER,reward_exp_raw INTEGER
);
CREATE TABLE supply_request_variants(
 request_id INTEGER NOT NULL REFERENCES supply_requests(id),group_id INTEGER NOT NULL REFERENCES supply_request_groups(id),
 PRIMARY KEY(request_id,group_id)
) WITHOUT ROWID;
CREATE TABLE journals(
 id INTEGER PRIMARY KEY,source_key TEXT,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 map_source_id INTEGER,name_en TEXT,name_localized TEXT,race_id SMALLINT REFERENCES races(id),display_order INTEGER,
 default_map_x_raw REAL,default_map_y_raw REAL,default_zoom_raw REAL
);
