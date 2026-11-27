-- Source-backed world definitions. No live presence or level-derived combat stats.
CREATE TABLE world_maps(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 source_name TEXT,name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 type_id SMALLINT REFERENCES catalogue_codes(id),race_id SMALLINT REFERENCES races(id),
 base_map_reference INTEGER,base_map_id INTEGER REFERENCES world_maps(id),
 default_subzone_reference INTEGER,default_subzone_id INTEGER REFERENCES world_subzones(id),
 base_directory TEXT,filename TEXT,entrance_name TEXT
);
CREATE TABLE world_subzones(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),
 zone_reference INTEGER,type_id SMALLINT REFERENCES catalogue_codes(id),pk_type_id SMALLINT REFERENCES catalogue_codes(id),
 world_map_area TEXT,world_map_name_actor TEXT,map_enabled BOOLEAN CHECK(map_enabled IN(0,1))
);
CREATE INDEX world_subzones_map ON world_subzones(map_id);
CREATE TABLE world_dungeon_groups(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 start_ticks_raw INTEGER,display_order INTEGER,shown BOOLEAN CHECK(shown IN(0,1))
);
CREATE TABLE world_dungeons(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),linked_map_id INTEGER REFERENCES world_maps(id),
 group_id INTEGER REFERENCES world_dungeon_groups(id),type_id SMALLINT REFERENCES catalogue_codes(id),
 subtype_id SMALLINT REFERENCES catalogue_codes(id),difficulty_id SMALLINT REFERENCES catalogue_codes(id),
 recommended_level INTEGER,recommended_min_members INTEGER,recommended_max_members INTEGER,
 cooldown_raw INTEGER,death_limit INTEGER,rebirth_limit INTEGER,
 thumbnail_icon_id SMALLINT REFERENCES catalogue_icons(id),background_icon_id SMALLINT REFERENCES catalogue_icons(id)
);
CREATE INDEX world_dungeons_map ON world_dungeons(map_id);
CREATE TABLE world_dungeon_levels(
 id INTEGER PRIMARY KEY,dungeon_reference TEXT,dungeon_id INTEGER REFERENCES world_dungeons(id),
 group_reference TEXT,level INTEGER,equipment_level INTEGER,unlock_type_id SMALLINT REFERENCES catalogue_codes(id)
);
CREATE TABLE world_ui_regions(
 id INTEGER PRIMARY KEY,map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),
 subzone_reference INTEGER,subzone_id INTEGER REFERENCES world_subzones(id),dungeon_reference INTEGER,dungeon_id INTEGER REFERENCES world_dungeons(id),
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 type_id SMALLINT REFERENCES catalogue_codes(id),icon_type_id SMALLINT REFERENCES catalogue_codes(id),actor_name TEXT,
 enabled BOOLEAN CHECK(enabled IN(0,1)),teleport_available BOOLEAN CHECK(teleport_available IN(0,1))
);
CREATE INDEX world_ui_regions_map ON world_ui_regions(map_id);
CREATE TABLE world_map_categories(
 map_id INTEGER REFERENCES world_maps(id),map_reference INTEGER NOT NULL,position INTEGER NOT NULL,
 category_id SMALLINT REFERENCES catalogue_codes(id),name_en TEXT,name_localized TEXT,PRIMARY KEY(map_reference,position)
) WITHOUT ROWID;
CREATE TABLE world_npcs(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 type_id SMALLINT REFERENCES catalogue_codes(id),subtype_id SMALLINT REFERENCES catalogue_codes(id),
 relationship_id SMALLINT REFERENCES catalogue_codes(id),creature_type_id SMALLINT REFERENCES catalogue_codes(id),
 level INTEGER,grade INTEGER,tribe TEXT,resource_key TEXT,
 level_scale_type_id SMALLINT REFERENCES catalogue_codes(id),world_level_scale_type_id SMALLINT REFERENCES catalogue_codes(id),
 attackable BOOLEAN CHECK(attackable IN(0,1)),named BOOLEAN CHECK(named IN(0,1))
);
CREATE TABLE world_npc_skills(
 npc_id INTEGER NOT NULL REFERENCES world_npcs(id),position INTEGER NOT NULL,skill_reference INTEGER NOT NULL,
 skill_id INTEGER REFERENCES skills(id),PRIMARY KEY(npc_id,position)
) WITHOUT ROWID;
CREATE TABLE world_npc_loot(
 npc_id INTEGER PRIMARY KEY REFERENCES world_npcs(id),loot_table_id INTEGER NOT NULL REFERENCES loot_tables(id),
 soul_item_reference INTEGER,soul_item_id INTEGER REFERENCES items(id)
);
CREATE TABLE world_environment_objects(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 category_id SMALLINT REFERENCES catalogue_codes(id),usage_id SMALLINT REFERENCES catalogue_codes(id),resource_key TEXT,
 level INTEGER,item_level INTEGER,interaction_time_raw INTEGER,interaction_range_raw INTEGER,
 attackable BOOLEAN CHECK(attackable IN(0,1)),gather_source_id INTEGER REFERENCES world_gather_sources(id)
);
CREATE TABLE world_gather_sources(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,type_id SMALLINT REFERENCES catalogue_codes(id),grade_id SMALLINT REFERENCES catalogue_codes(id),
 mastery_level INTEGER,material_group_reference TEXT,quest_step_reference INTEGER,quest_step_id INTEGER REFERENCES quest_steps(id),
 conversion_recipe_source TEXT CHECK(conversion_recipe_source='ConversionRecipe'),conversion_recipe_id INTEGER,
 FOREIGN KEY(conversion_recipe_source,conversion_recipe_id) REFERENCES recipes(source_name,id)
);
CREATE TABLE world_gather_materials(
 source_id INTEGER NOT NULL REFERENCES world_gather_sources(id),material_key INTEGER NOT NULL,
 gather_gauge_raw INTEGER,PRIMARY KEY(source_id,material_key)
) WITHOUT ROWID;
CREATE TABLE world_gather_results(
 source_id INTEGER NOT NULL REFERENCES world_gather_sources(id),material_key INTEGER NOT NULL,role TEXT NOT NULL CHECK(role IN('main','additional','failure','quest')),
 position INTEGER NOT NULL,item_reference TEXT NOT NULL,item_id INTEGER REFERENCES items(id),quantity INTEGER,
 PRIMARY KEY(source_id,material_key,role,position)
) WITHOUT ROWID;
CREATE INDEX world_gather_results_item ON world_gather_results(item_id);
CREATE TABLE world_gather_mastery(
 level INTEGER PRIMARY KEY,grade_id SMALLINT REFERENCES catalogue_codes(id),required_exp INTEGER,
 bonus_skill_points INTEGER,bonus_specialized_points INTEGER
);
CREATE TABLE world_gather_skills(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 name_en TEXT,name_localized TEXT,description_en TEXT,description_localized TEXT,
 category_id SMALLINT REFERENCES catalogue_codes(id),condition_type_id SMALLINT REFERENCES catalogue_codes(id),display_order INTEGER
);
CREATE TABLE world_gather_effects(
 id INTEGER PRIMARY KEY,skill_reference TEXT,skill_id INTEGER REFERENCES world_gather_skills(id),level INTEGER,
 type_id SMALLINT REFERENCES catalogue_codes(id),active_probability_raw INTEGER,skill_points INTEGER,specialized_points INTEGER
);
CREATE TABLE world_periodic_spawns(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),source_name TEXT,
 map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),spawner_name TEXT,type_id SMALLINT REFERENCES catalogue_codes(id),
 cycle_raw INTEGER,time_ticks_raw INTEGER,
 monday BOOLEAN CHECK(monday IN(0,1)),tuesday BOOLEAN CHECK(tuesday IN(0,1)),wednesday BOOLEAN CHECK(wednesday IN(0,1)),
 thursday BOOLEAN CHECK(thursday IN(0,1)),friday BOOLEAN CHECK(friday IN(0,1)),saturday BOOLEAN CHECK(saturday IN(0,1)),sunday BOOLEAN CHECK(sunday IN(0,1))
);
CREATE TABLE world_field_named(
 id INTEGER PRIMARY KEY,map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),source_name TEXT,spawner_name TEXT,
 period_spawn_reference INTEGER,period_spawn_id INTEGER REFERENCES world_periodic_spawns(id),type_id SMALLINT REFERENCES catalogue_codes(id)
);
-- Resource identity is its relative path, not an invented map definition.
CREATE TABLE world_map_assets(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 source_path TEXT NOT NULL UNIQUE,kind TEXT NOT NULL,map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),
 payload_sha256 TEXT NOT NULL,coordinate_state TEXT NOT NULL CHECK(coordinate_state='source'),
 transform_state TEXT NOT NULL CHECK(transform_state='uninterpreted')
);
CREATE INDEX world_map_assets_map ON world_map_assets(map_id);
CREATE TABLE world_spawners(
 id INTEGER PRIMARY KEY,map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),source_position INTEGER NOT NULL,
 source_name TEXT,unique_key INTEGER,maintaining_tag TEXT,single_fixed_rotation BOOLEAN CHECK(single_fixed_rotation IN(0,1)),
 dynamic_terrain BOOLEAN CHECK(dynamic_terrain IN(0,1)),UNIQUE(map_asset_id,source_position)
);
CREATE INDEX world_spawners_asset ON world_spawners(map_asset_id,source_name);
CREATE TABLE world_spawn_candidates(
 spawner_id INTEGER NOT NULL REFERENCES world_spawners(id),kind TEXT NOT NULL CHECK(kind IN('npc','environment')),
 position INTEGER NOT NULL,source_reference INTEGER NOT NULL,npc_id INTEGER REFERENCES world_npcs(id),
 environment_object_id INTEGER REFERENCES world_environment_objects(id),PRIMARY KEY(spawner_id,kind,position)
) WITHOUT ROWID;
CREATE INDEX world_spawn_candidates_npc ON world_spawn_candidates(npc_id);
CREATE INDEX world_spawn_candidates_environment ON world_spawn_candidates(environment_object_id);
CREATE TABLE world_spawn_positions(
 spawner_id INTEGER NOT NULL REFERENCES world_spawners(id),position INTEGER NOT NULL,
 x_raw REAL,y_raw REAL,z_raw REAL,pitch_raw REAL,yaw_raw REAL,roll_raw REAL,PRIMARY KEY(spawner_id,position)
) WITHOUT ROWID;
CREATE TABLE world_routes(
 id INTEGER PRIMARY KEY,map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),source_key TEXT NOT NULL,
 type_id SMALLINT REFERENCES catalogue_codes(id),traverse_type_id SMALLINT REFERENCES catalogue_codes(id),
 source_uid TEXT,move_type INTEGER,base_move_speed_raw REAL,UNIQUE(map_asset_id,source_key)
);
CREATE TABLE world_route_points(
 route_id INTEGER NOT NULL REFERENCES world_routes(id),position INTEGER NOT NULL,x_raw REAL,y_raw REAL,z_raw REAL,
 PRIMARY KEY(route_id,position)
) WITHOUT ROWID;
CREATE TABLE world_subzone_volumes(
 id INTEGER PRIMARY KEY,map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),source_key TEXT NOT NULL,
 subzone_reference INTEGER,subzone_id INTEGER REFERENCES world_subzones(id),
 x_raw REAL,y_raw REAL,z_raw REAL,height_min_raw REAL,height_max_raw REAL,radius_squared_raw REAL,UNIQUE(map_asset_id,source_key)
);
CREATE TABLE world_volume_points(
 volume_id INTEGER NOT NULL REFERENCES world_subzone_volumes(id),position INTEGER NOT NULL,x_raw REAL,y_raw REAL,
 PRIMARY KEY(volume_id,position)
) WITHOUT ROWID;
CREATE TABLE world_hierarchy_entries(
 map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),position INTEGER NOT NULL,
 map_reference INTEGER,map_id INTEGER REFERENCES world_maps(id),subzone_reference INTEGER,subzone_id INTEGER REFERENCES world_subzones(id),
 npc_type_ordinal INTEGER,npc_reference INTEGER,npc_id INTEGER REFERENCES world_npcs(id),item_reference INTEGER,item_id INTEGER REFERENCES items(id),unknown_word INTEGER,
 PRIMARY KEY(map_asset_id,position)
) WITHOUT ROWID;
CREATE TABLE world_periodic_spawner_links(
 period_spawn_id INTEGER NOT NULL REFERENCES world_periodic_spawns(id),spawner_id INTEGER NOT NULL REFERENCES world_spawners(id),
 PRIMARY KEY(period_spawn_id,spawner_id)
) WITHOUT ROWID;
CREATE TABLE world_event_spawner_commands(
 map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),source_path TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('StartSpawner','EndSpawner')),spawner_reference TEXT,
 PRIMARY KEY(map_asset_id,source_path)
) WITHOUT ROWID;
CREATE TABLE world_event_spawner_links(
 map_asset_id INTEGER NOT NULL,source_path TEXT NOT NULL,spawner_id INTEGER NOT NULL REFERENCES world_spawners(id),
 PRIMARY KEY(map_asset_id,source_path,spawner_id),
 FOREIGN KEY(map_asset_id,source_path) REFERENCES world_event_spawner_commands(map_asset_id,source_path)
) WITHOUT ROWID;
CREATE TABLE world_map_layouts(
 map_asset_id INTEGER PRIMARY KEY REFERENCES world_map_assets(id),table_key_reference TEXT,
 candidate_map_id INTEGER REFERENCES world_maps(id),sector_columns INTEGER,sector_rows INTEGER,
 sector_plane_size_raw REAL,sector_texture_size INTEGER,sector_world_size_raw REAL,start_x_raw REAL,start_y_raw REAL,
 min_x_raw REAL,min_y_raw REAL,min_z_raw REAL,max_x_raw REAL,max_y_raw REAL,max_z_raw REAL,
 bounds_valid BOOLEAN CHECK(bounds_valid IN(0,1)),interpretation_state TEXT NOT NULL CHECK(interpretation_state='candidate')
);
CREATE TABLE world_map_preview_locations(
 map_asset_id INTEGER NOT NULL REFERENCES world_map_assets(id),source_key TEXT NOT NULL,x_raw REAL,y_raw REAL,z_raw REAL,
 PRIMARY KEY(map_asset_id,source_key)
) WITHOUT ROWID;
CREATE TABLE world_dungeon_loot(
 dungeon_id INTEGER NOT NULL REFERENCES world_dungeons(id),loot_table_id INTEGER NOT NULL REFERENCES loot_tables(id),
 PRIMARY KEY(dungeon_id,loot_table_id)
) WITHOUT ROWID;
