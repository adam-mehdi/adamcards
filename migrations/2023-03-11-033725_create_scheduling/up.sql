CREATE TABLE Quotas (
    id                 INTEGER,
    days_to_go         INTEGER,
    new_assigned       INTEGER NOT NULL,
    review_assigned    INTEGER NOT NULL,
    new_quota_initial  INTEGER NOT NULL CHECK (new_quota_initial > -1),
    review_quota_initial INTEGER NOT NULL CHECK (review_quota_initial > -1),
    new_practiced      INTEGER NOT NULL CHECK (new_practiced > -1),
    review_practiced   INTEGER NOT NULL,
    PRIMARY KEY (id, days_to_go),
    FOREIGN KEY (id) REFERENCES Decks (id) ON DELETE CASCADE
);
