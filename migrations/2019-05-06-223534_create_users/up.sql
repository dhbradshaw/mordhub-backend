CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  steam_id BIGINT UNIQUE NOT NULL
);

CREATE TABLE images (
  id SERIAL PRIMARY KEY,
  url VARCHAR(50) NOT NULL,
  uploaded_by INTEGER NOT NULL REFERENCES users(id),
  upload_date DATE NOT NULL DEFAULT CURRENT_DATE
);

CREATE TABLE loadouts (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id),
  title VARCHAR(80) NOT NULL,
  main_image_id INTEGER NOT NULL REFERENCES images(id)
);
