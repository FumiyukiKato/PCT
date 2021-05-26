-- Your SQL goes here
CREATE TABLE trajectory (
  id INTEGER NOT NULL PRIMARY KEY,
  time UNSIGNED BIG INT NOT NULL,
  longitude FLOAT NOT NULL,
  latitude FLOAT NOT NULL
);

CREATE INDEX all_range_index ON trajectory (time, longitude, latitude);
CREATE INDEX time_index ON trajectory (time);