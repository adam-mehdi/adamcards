CREATE TABLE Entries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            VARCHAR(30) NOT NULL CHECK (name <> ''),
    is_expanded     BOOLEAN
);

CREATE TABLE Folders ( 
    id INTEGER PRIMARY KEY REFERENCES Entries (id) ON DELETE CASCADE
);

CREATE TABLE Deadlines ( 
    id              INTEGER PRIMARY KEY,
    date_created    TIMESTAMP NOT NULL,
    deadline_date   TIMESTAMP NOT NULL,
    study_intensity INTEGER,
    num_reset       INTEGER NOT NULL CHECK (num_reset > -1),
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE
);

CREATE TABLE Decks (
    id              INTEGER PRIMARY KEY,
    date_created    TIMESTAMP NOT NULL,
    num_boxes       INTEGER NOT NULL,
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE
);

CREATE TABLE Parents (
    parent_id       INTEGER, 
    child_id        INTEGER,
    PRIMARY KEY (parent_id, child_id),
    FOREIGN KEY (parent_id) REFERENCES Entries (id) ON DELETE CASCADE,
    FOREIGN KEY (child_id) REFERENCES Entries (id) ON DELETE CASCADE
);
