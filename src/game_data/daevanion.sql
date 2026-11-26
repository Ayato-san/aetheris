-- Definitions and user-confirmed orthogonal connectivity; no player ownership.
CREATE TABLE daevanion_point_types(
 id SMALLINT PRIMARY KEY CHECK(typeof(id)='integer' AND id BETWEEN 1 AND 32767),code TEXT NOT NULL UNIQUE,
 source_name TEXT,max_value_raw INTEGER,currency_id SMALLINT REFERENCES catalogue_codes(id)
) WITHOUT ROWID;
CREATE TABLE daevanion_level_points(
 character_level INTEGER NOT NULL,position INTEGER NOT NULL,point_type_id SMALLINT REFERENCES daevanion_point_types(id),points_raw INTEGER,
 PRIMARY KEY(character_level,position)
) WITHOUT ROWID;
CREATE TABLE daevanion_boards(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),
 name_en TEXT,name_localized TEXT,buff_title_en TEXT,buff_title_localized TEXT,buff_description_en TEXT,buff_description_localized TEXT,
 class_id SMALLINT REFERENCES skill_classes(id),required_level INTEGER,point_type_reference TEXT,point_type_id SMALLINT REFERENCES daevanion_point_types(id),
 reset_currency_id SMALLINT REFERENCES catalogue_codes(id),display_order INTEGER
);
CREATE TABLE daevanion_nodes(
 id INTEGER PRIMARY KEY,client_version_id SMALLINT NOT NULL REFERENCES client_version(id),board_reference INTEGER,board_id INTEGER REFERENCES daevanion_boards(id),
 row_number INTEGER,column_number INTEGER,type_id SMALLINT REFERENCES catalogue_codes(id),grade_id SMALLINT REFERENCES catalogue_codes(id),
 required_level INTEGER,point_cost_raw INTEGER,reset_cost_raw INTEGER,
 name_en TEXT,name_localized TEXT,start_text_en TEXT,start_text_localized TEXT,
 condition_type_id SMALLINT REFERENCES catalogue_codes(id),condition_value_source TEXT,
 auto_learn BOOLEAN CHECK(auto_learn IN(0,1)),UNIQUE(board_id,row_number,column_number)
);
CREATE INDEX daevanion_nodes_board ON daevanion_nodes(board_id,type_id);
-- Each undirected connection is stored once; None cells are not projected.
CREATE TABLE daevanion_node_connections(
 node_id INTEGER NOT NULL REFERENCES daevanion_nodes(id),neighbour_id INTEGER NOT NULL REFERENCES daevanion_nodes(id),
 CHECK(node_id<neighbour_id),PRIMARY KEY(node_id,neighbour_id)
) WITHOUT ROWID;
CREATE INDEX daevanion_connections_reverse ON daevanion_node_connections(neighbour_id,node_id);
CREATE TABLE daevanion_node_parameters(
 node_id INTEGER NOT NULL REFERENCES daevanion_nodes(id),position INTEGER NOT NULL CHECK(position BETWEEN 1 AND 10),value_source TEXT,
 PRIMARY KEY(node_id,position)
) WITHOUT ROWID;
CREATE TABLE daevanion_stat_parameters(
 node_id INTEGER NOT NULL REFERENCES daevanion_nodes(id),parameter_position INTEGER NOT NULL CHECK(parameter_position IN(1,3,5,7,9)),
 stat_reference TEXT NOT NULL,stat_id SMALLINT REFERENCES stats(id),value_source TEXT,value_integer_raw INTEGER,
 interpretation_state TEXT NOT NULL CHECK(interpretation_state='candidate'),
 PRIMARY KEY(node_id,parameter_position)
) WITHOUT ROWID;
CREATE TABLE daevanion_skill_parameters(
 node_id INTEGER PRIMARY KEY REFERENCES daevanion_nodes(id),skill_reference_source TEXT NOT NULL,skill_reference INTEGER,
 skill_id INTEGER REFERENCES skills(id),value_source TEXT,level_bonus INTEGER,
 interpretation_state TEXT NOT NULL CHECK(interpretation_state='verified')
);
CREATE INDEX daevanion_skill_target ON daevanion_skill_parameters(skill_id);
CREATE TABLE daevanion_point_items(
 item_id INTEGER PRIMARY KEY REFERENCES items(id),effect_type_id SMALLINT REFERENCES catalogue_codes(id),point_reference TEXT,
 point_type_id SMALLINT REFERENCES daevanion_point_types(id),value_source TEXT,value_integer_raw INTEGER,
 interpretation_state TEXT NOT NULL CHECK(interpretation_state='candidate')
);
