CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT,
  plan_type TEXT,
  telegram_id INTEGER UNIQUE
);

CREATE TABLE websites (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  last_checked_time TEXT,
  status INTEGER,
  url TEXT UNIQUE
);

CREATE TABLE user_websites (
  user_id INTEGER, 
  website_id INTEGER,
  
  FOREIGN KEY (user_id) REFERENCES users(id),
  FOREIGN KEY (website_id) REFERENCES websites(id),
  
  PRIMARY KEY (user_id, website_id)
);
