CREATE TABLE IF NOT EXISTS title_master (
    tid INT PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS song_select_master (
    `group` INT PRIMARY KEY,
    start_song_time INT NOT NULL,
    song_time INT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_master (
    sid INT PRIMARY KEY,
    `group` INT NOT NULL,
    difficulty INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    composer VARCHAR(255) NOT NULL,
    start_offset FLOAT NOT NULL,
    bg INT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_score_rate_master (
    r_type INT PRIMARY KEY,
    rate FLOAT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_base_score_master (
    score INT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_judge_zone_master (
    j_type INT PRIMARY KEY,
    zone FLOAT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_base_hp_master (
    hp INT NOT NULL
);
CREATE TABLE IF NOT EXISTS song_hp_rate_master (
    j_type INT PRIMARY KEY,
    rate INT NOT NULL
);
CREATE TABLE IF NOT EXISTS sound_sheet_name_master (
    category INT NOT NULL,
    id INT NOT NULL,
    sheet_name VARCHAR(255) NOT NULL,
    cue_name VARCHAR(255) NOT NULL,
    PRIMARY KEY (category, id)
);
CREATE TABLE IF NOT EXISTS result_master (
    rid INT PRIMARY KEY
);
