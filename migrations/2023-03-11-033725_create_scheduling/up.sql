-- Your SQL goes here
CREATE TABLE Quotas (
    id                 INT,
    days_to_go         INT,
    new_assigned       INT NOT NULL,
    review_assigned    INT NOT NULL,
    new_quota_initial  INT NOT NULL CHECK (new_quota_initial > -1),
    review_quota_initial INT NOT NULL CHECK (review_quota_initial > -1),
    new_practiced      INT NOT NULL CHECK (new_practiced > -1),
    review_practiced   INT NOT NULL,
    PRIMARY KEY (id, days_to_go),
    FOREIGN KEY (id) REFERENCES Decks (id) ON DELETE CASCADE
);