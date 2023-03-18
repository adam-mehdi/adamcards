-- Your SQL goes here

CREATE TABLE Entries (
    id              SERIAL PRIMARY KEY,
    name            VARCHAR(30) NOT NULL CHECK (name <> ''),
    is_expanded     BOOLEAN
);

CREATE TABLE Folders ( 
    id INT PRIMARY KEY REFERENCES Entries ON DELETE CASCADE
);

CREATE TABLE Deadlines ( 
    id              INT PRIMARY KEY,
    date_created    TIMESTAMPTZ NOT NULL,
    deadline_date   TIMESTAMPTZ NOT NULL,
    study_intensity INT,
    num_reset       INT NOT NULL CHECK (num_reset > -1),
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE
);


CREATE TABLE Decks (
    id              INT PRIMARY KEY,
    date_created    TIMESTAMPTZ NOT NULL,
    num_boxes       INT NOT NULL,
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE
);

CREATE TABLE Parents (
    parent_id       INT, 
    child_id        INT,
    PRIMARY KEY (parent_id, child_id),
    FOREIGN KEY (parent_id) REFERENCES Entries (id) ON DELETE CASCADE,
    FOREIGN KEY (child_id) REFERENCES Entries (id) ON DELETE CASCADE
);