-- Your SQL goes here

CREATE TABLE DeckItems (
	item_id        	SERIAL PRIMARY KEY,
	deck_id         INT NOT NULL,
	FOREIGN KEY (deck_id) REFERENCES Decks (id) ON DELETE CASCADE
);

CREATE TABLE Documents (
	id              SERIAL PRIMARY KEY,
	source_text     BYTEA,
	notes           TEXT,
	FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE
);
	
CREATE TABLE Cards(
	id              INT PRIMARY KEY,
	front           TEXT NOT NULL,
	back            TEXT NOT NULL,
	queue_score		INT,
	box_position    INT CHECK (box_position > -1) NOT NULL,
	FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE
);

CREATE TABLE Media (
	id SERIAL PRIMARY KEY,
	content BYTEA,
	entry_id INT,
	FOREIGN KEY (entry_id) REFERENCES DeckItems (item_id)
);

CREATE TABLE CardHistory (
	card_id              INT PRIMARY KEY NOT NULL,
	review_time          TIMESTAMPTZ NOT NULL,
	user_response        INT CHECK (user_response in (-1, 0, 1)),
	duration_to_respond  INT CHECK (duration_to_respond > 0),
	box_position_initial INT CHECK (box_position_initial > -1),
	FOREIGN KEY (card_id) REFERENCES Cards (id) ON DELETE CASCADE
);

CREATE TABLE UserConfig (
    config_id      SERIAL PRIMARY KEY,
	is_dark_mode   BOOLEAN NOT NULL,
	is_text_field  BOOLEAN NOT NULL
);