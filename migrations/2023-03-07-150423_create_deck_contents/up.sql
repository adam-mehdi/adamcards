-- Your SQL goes here
CREATE TABLE DeckItems (
	item_id        	INTEGER PRIMARY KEY AUTOINCREMENT,
	deck_id         INTEGER NOT NULL,
	FOREIGN KEY (deck_id) REFERENCES Decks (id) ON DELETE CASCADE
);

CREATE TABLE Documents (
	id              INTEGER PRIMARY KEY AUTOINCREMENT,
	source_text     BLOB,
	notes           TEXT,
	FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE
);
	
CREATE TABLE Cards(
	id              INTEGER PRIMARY KEY,
	front           TEXT NOT NULL,
	back            TEXT NOT NULL,
	queue_score		INTEGER,
	box_position    INTEGER CHECK (box_position > -1) NOT NULL,
	FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE
);

CREATE TABLE Media (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	content BLOB,
	entry_id INTEGER,
	FOREIGN KEY (entry_id) REFERENCES DeckItems (item_id) ON DELETE SET NULL
);

CREATE TABLE CardHistory (
	card_id              INTEGER PRIMARY KEY NOT NULL,
	review_time          TIMESTAMP NOT NULL,
	user_response        INTEGER CHECK (user_response in (-1, 0, 1)),
	duration_to_respond  INTEGER CHECK (duration_to_respond > 0),
	box_position_initial INTEGER CHECK (box_position_initial > -1),
	FOREIGN KEY (card_id) REFERENCES Cards (id) ON DELETE CASCADE
);

CREATE TABLE UserConfig (
    config_id      INTEGER PRIMARY KEY AUTOINCREMENT,
	is_dark_mode   BOOLEAN NOT NULL,
	is_text_field  BOOLEAN NOT NULL
);