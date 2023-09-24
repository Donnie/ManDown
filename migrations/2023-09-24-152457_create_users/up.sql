CREATE TABLE users (
  user_id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT,
  plan_type TEXT,
  telegram_id TEXT
);

CREATE TABLE websites (
  website_id INTEGER PRIMARY KEY AUTOINCREMENT,
  status INTEGER,
  url UNIQUE
);

CREATE TABLE user_websites (
  user_id INTEGER, 
  website_id INTEGER,
  last_checked_time TEXT,
  
  FOREIGN KEY (user_id) REFERENCES users(user_id),
  FOREIGN KEY (website_id) REFERENCES websites(website_id),
  
  PRIMARY KEY (user_id, website_id)
);
