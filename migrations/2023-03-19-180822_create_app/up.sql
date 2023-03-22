-- Your SQL goes here

CREATE TABLE entries (                                                          
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                          
    name            VARCHAR(30) NOT NULL CHECK (name <> ''),                    
    is_expanded     BOOLEAN                                                     
);                                                                              
                                                                                
CREATE TABLE folders (                                                          
    id INTEGER PRIMARY KEY NOT NULL REFERENCES Entries (id) ON DELETE CASCADE            
);                                                                              
                                                                                
CREATE TABLE deadlines (                                                        
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    date_created    TIMESTAMP NOT NULL,                                         
    deadline_date   TIMESTAMP NOT NULL,                                         
    study_intensity INTEGER,                                                    
    num_reset       INTEGER NOT NULL CHECK (num_reset > -1),                    
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE                  
);                                                                              
                                                                                
CREATE TABLE decks (                                                            
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    date_created    TIMESTAMP NOT NULL,                                         
    num_boxes       INTEGER NOT NULL,                                           
    FOREIGN KEY (id) REFERENCES Entries (id) ON DELETE CASCADE                  
);                                                                              
                                                                                
CREATE TABLE parents (                                                          
    parent_id       INTEGER NOT NULL,                                                    
    child_id        INTEGER NOT NULL,                                                    
    PRIMARY KEY (parent_id, child_id),                                          
    FOREIGN KEY (parent_id) REFERENCES Entries (id) ON DELETE CASCADE,          
    FOREIGN KEY (child_id) REFERENCES Entries (id) ON DELETE CASCADE            
);                                                                              




CREATE TABLE deckitems (                                                        
    item_id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                          
    deck_id         INTEGER NOT NULL,                                           
    FOREIGN KEY (deck_id) REFERENCES Decks (id) ON DELETE CASCADE               
);                                                                              
                                                                                
CREATE TABLE documents (                                                        
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                          
    source_text     BLOB,                                                       
    notes           TEXT,                                                       
    FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE           
);                                                                              
                                                                                
CREATE TABLE cards(                                                             
    id              INTEGER PRIMARY KEY NOT NULL,                                        
    front           TEXT NOT NULL,                                              
    back            TEXT NOT NULL,                                              
    queue_score     INTEGER,                                                    
    box_position    INTEGER CHECK (box_position > -1) NOT NULL,                 
    FOREIGN KEY (id) REFERENCES DeckItems (item_id) ON DELETE CASCADE           
);                                                                              
                                                                                
CREATE TABLE media (                                                            
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,                                       
    content BLOB,                                                               
    entry_id INTEGER,                                                           
    FOREIGN KEY (entry_id) REFERENCES DeckItems (item_id) ON DELETE SET NULL    
);                                                                              
                                                                                
CREATE TABLE cardhistory (                                                      
    card_id              INTEGER PRIMARY KEY NOT NULL,                          
    review_time          TIMESTAMP NOT NULL,                                    
    user_response        INTEGER CHECK (user_response in (-1, 0, 1)),           
    duration_to_respond  INTEGER CHECK (duration_to_respond > 0),               
    box_position_initial INTEGER CHECK (box_position_initial > -1),             
    FOREIGN KEY (card_id) REFERENCES Cards (id) ON DELETE CASCADE               
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
      