-- Source IDs are scoped by their table; conversion and crafting IDs may overlap.
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
 resolution_state TEXT NOT NULL CHECK(resolution_state IN('resolved','missing','ambiguous','not_researched')),
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
CREATE TABLE recipes(
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),id INTEGER NOT NULL CHECK(id>0),source_key TEXT NOT NULL,
 client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,
 type_id SMALLINT REFERENCES catalogue_codes(id),category_id SMALLINT REFERENCES catalogue_codes(id),
 subcategory_id SMALLINT REFERENCES catalogue_codes(id),race_id SMALLINT REFERENCES races(id),
 mastery_level INTEGER,combo_reference TEXT,combo_id INTEGER,combo_probability_raw INTEGER,rate_group_reference TEXT,
 show_recipe BOOLEAN CHECK(show_recipe IN(0,1)),guild_craft BOOLEAN CHECK(guild_craft IN(0,1)),
 PRIMARY KEY(source_name,id),UNIQUE(source_name,source_key),
 FOREIGN KEY(source_name,combo_id) REFERENCES recipes(source_name,id)
) WITHOUT ROWID;
CREATE INDEX recipes_names ON recipes(name_localized,name_en);
CREATE TABLE recipe_results(
 source_name TEXT NOT NULL,recipe_id INTEGER NOT NULL,role TEXT NOT NULL,position INTEGER NOT NULL,
 item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),quantity INTEGER,enchantment INTEGER,probability_raw INTEGER,
 PRIMARY KEY(source_name,recipe_id,role,position),
 FOREIGN KEY(source_name,recipe_id) REFERENCES recipes(source_name,id)
) WITHOUT ROWID;
CREATE INDEX recipe_results_item ON recipe_results(item_id);
CREATE TABLE recipe_components(
 source_name TEXT NOT NULL,recipe_id INTEGER NOT NULL,position INTEGER NOT NULL,
 item_reference TEXT,item_id INTEGER REFERENCES items(id),quantity INTEGER,enchantment INTEGER,
 replacement_reference TEXT,combine_reference TEXT,
 PRIMARY KEY(source_name,recipe_id,position),
 FOREIGN KEY(source_name,recipe_id) REFERENCES recipes(source_name,id)
) WITHOUT ROWID;
CREATE INDEX recipe_components_item ON recipe_components(item_id);
CREATE TABLE recipe_component_options(
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),group_reference TEXT NOT NULL,role TEXT NOT NULL,position INTEGER NOT NULL,
 item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),quantity INTEGER,enchantment INTEGER,
 PRIMARY KEY(source_name,group_reference,role,position)
) WITHOUT ROWID;
CREATE INDEX recipe_options_item ON recipe_component_options(item_id);
CREATE TABLE recipe_costs(
 source_name TEXT NOT NULL,recipe_id INTEGER NOT NULL,kind TEXT NOT NULL,
 currency_id SMALLINT REFERENCES catalogue_codes(id),amount_raw INTEGER NOT NULL,
 PRIMARY KEY(source_name,recipe_id,kind),
 FOREIGN KEY(source_name,recipe_id) REFERENCES recipes(source_name,id)
) WITHOUT ROWID;
CREATE TABLE recipe_rates(
 group_reference TEXT NOT NULL,position INTEGER NOT NULL,gap INTEGER,rate_raw INTEGER,
 PRIMARY KEY(group_reference,position)
) WITHOUT ROWID;
CREATE TABLE quest_parts(
 id INTEGER PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,part_order INTEGER,name_en TEXT,name_localized TEXT
);
CREATE TABLE quests(
 id INTEGER PRIMARY KEY CHECK(id>0),source_key TEXT NOT NULL UNIQUE,
 client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 type_id SMALLINT REFERENCES catalogue_codes(id),grade_id SMALLINT REFERENCES catalogue_codes(id),race_id SMALLINT REFERENCES races(id),
 required_level INTEGER,recommended_level INTEGER,repeat_type_id SMALLINT REFERENCES catalogue_codes(id),
 part_reference TEXT,part_id INTEGER REFERENCES quest_parts(id),next_quest_reference TEXT,next_quest_id INTEGER REFERENCES quests(id),
 can_give_up BOOLEAN CHECK(can_give_up IN(0,1))
);
CREATE INDEX quests_names ON quests(name_localized,name_en);
CREATE TABLE quest_texts(
 quest_id INTEGER NOT NULL REFERENCES quests(id),source_path TEXT NOT NULL,string_key TEXT NOT NULL,
 text_en TEXT,text_localized TEXT,
 PRIMARY KEY(quest_id,source_path)
) WITHOUT ROWID;
CREATE TABLE quest_steps(
 id INTEGER PRIMARY KEY CHECK(id>0),quest_reference TEXT NOT NULL,quest_id INTEGER REFERENCES quests(id),step_order INTEGER,
 fail_condition_id SMALLINT REFERENCES catalogue_codes(id),fail_go_to_order INTEGER,fail_go_to_step_id INTEGER REFERENCES quest_steps(id),
 unified_objectives BOOLEAN CHECK(unified_objectives IN(0,1)),unified_objective_score INTEGER
);
CREATE INDEX quest_steps_quest ON quest_steps(quest_id,step_order);
CREATE TABLE quest_objectives(
 step_id INTEGER NOT NULL REFERENCES quest_steps(id),position INTEGER NOT NULL,
 type_id SMALLINT REFERENCES catalogue_codes(id),values_json TEXT NOT NULL CHECK(json_valid(values_json)),map_source_id INTEGER,
 optional BOOLEAN CHECK(optional IN(0,1)),hidden BOOLEAN CHECK(hidden IN(0,1)),marker BOOLEAN CHECK(marker IN(0,1)),
 PRIMARY KEY(step_id,position)
) WITHOUT ROWID;
-- A reward group can contain several rows and class-specific alternatives.
-- Matching its numeric text to Quest.ID is retained as a candidate relation.
CREATE TABLE quest_reward_groups(
 source_name TEXT NOT NULL REFERENCES catalogue_sources(name),source_row INTEGER NOT NULL,group_reference TEXT NOT NULL,
 quest_id INTEGER REFERENCES quests(id),class_id SMALLINT REFERENCES catalogue_codes(id),experience INTEGER,random_reward_reference TEXT,
 PRIMARY KEY(source_name,source_row)
) WITHOUT ROWID;
CREATE INDEX quest_reward_quest ON quest_reward_groups(quest_id);
CREATE TABLE quest_rewards(
 source_name TEXT NOT NULL,source_row INTEGER NOT NULL,mode TEXT NOT NULL,position INTEGER NOT NULL,
 item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),quantity INTEGER,enchantment INTEGER,
 PRIMARY KEY(source_name,source_row,mode,position),
 FOREIGN KEY(source_name,source_row) REFERENCES quest_reward_groups(source_name,source_row)
) WITHOUT ROWID;
CREATE INDEX quest_rewards_item ON quest_rewards(item_id);
