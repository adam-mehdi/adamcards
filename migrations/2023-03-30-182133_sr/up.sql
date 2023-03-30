-- Your SQL goes here
-- Your SQL goes here

CREATE TABLE entries (                                                          
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                          
    name            VARCHAR(30) NOT NULL CHECK (name <> ''),                    
    is_expanded     BOOLEAN                                                     
);                                                                              
                                                                                
CREATE TABLE folders (                                                          
    id INTEGER PRIMARY KEY NOT NULL REFERENCES entries (id) ON DELETE CASCADE            
);                                                                              
                                                                                
CREATE TABLE deadlines (                                                        
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    deadline_date   TIMESTAMP,                                         
    study_intensity INTEGER,                                                    
    num_reset       INTEGER CHECK (num_reset > -1),                    
    is_anki         BOOLEAN NOT NULL,
    FOREIGN KEY (id) REFERENCES entries (id) ON DELETE CASCADE                  
);                                                                              
                                                                                
CREATE TABLE decks (                                                            
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    num_boxes       INTEGER,                                           
    new_per_day     INTEGER,
    FOREIGN KEY (id) REFERENCES entries (id) ON DELETE CASCADE                  
);                                                                              
                                                                                
CREATE TABLE parents (                                                          
    parent_id       INTEGER NOT NULL,                                                    
    child_id        INTEGER NOT NULL,                                                    
    PRIMARY KEY (parent_id, child_id),                                          
    FOREIGN KEY (parent_id) REFERENCES entries (id) ON DELETE CASCADE,          
    FOREIGN KEY (child_id) REFERENCES entries (id) ON DELETE CASCADE            
);                                                                              
                                                                                
CREATE TABLE cards(                                                             
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    deck_id         INTEGER NOT NULL,
    front           TEXT NOT NULL,                                              
    back            TEXT NOT NULL,                                              
    queue_score     INTEGER,                                                    
    -- AM-1 field
    box_position    INTEGER CHECK (box_position > -1),                 
    -- Anki fields
    repetitions     INTEGER,
    easiness        REAL,
    interval        INTEGER,
    next_practice   DATE,
    -- AI fields
    rephrasing1     TEXT,
    rephrasing2     TEXT,
    rephrasing3     TEXT,
    rephrasing4     TEXT,
    rephrasing5     TEXT,
    explanation     TEXT,

    FOREIGN KEY (deck_id) REFERENCES decks (id) ON DELETE CASCADE           
);                                                                              
                                                                                
CREATE TABLE userconfig (                                                       
    config_id      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                           
    is_dark_mode   BOOLEAN NOT NULL,                                            
    is_text_field  BOOLEAN NOT NULL                                             
);                                                                              
                                                                                                    
CREATE TABLE quotas (                                                           
    id                 INTEGER NOT NULL,                                                 
    days_to_go         INTEGER NOT NULL,                                                 
    new_assigned       INTEGER NOT NULL,                                        
    review_assigned    INTEGER NOT NULL,                                        
    new_quota_initial  INTEGER NOT NULL CHECK (new_quota_initial > -1),         
    review_quota_initial INTEGER NOT NULL CHECK (review_quota_initial > -1),    
    new_practiced      INTEGER NOT NULL CHECK (new_practiced > -1),             
    review_practiced   INTEGER NOT NULL,                                        
    PRIMARY KEY (id, days_to_go),                                               
    FOREIGN KEY (id) REFERENCES Decks (id) ON DELETE CASCADE                    
);                                                                              

CREATE TABLE ankiquotas (
    deck_id           INTEGER NOT NULL,
    date_practiced    DATE NOT NULL,
    new_practiced     INTEGER NOT NULL,
    review_practiced  INTEGER NOT NULL,
    PRIMARY KEY (deck_id, date_practiced)
    FOREIGN KEY (deck_id) REFERENCES decks(id)
);