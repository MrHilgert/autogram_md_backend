CREATE TABLE car_makes (
    id      SERIAL PRIMARY KEY,
    name    VARCHAR(100) NOT NULL UNIQUE,
    slug    VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE car_models (
    id      SERIAL PRIMARY KEY,
    make_id INTEGER NOT NULL REFERENCES car_makes(id) ON DELETE CASCADE,
    name    VARCHAR(100) NOT NULL,
    slug    VARCHAR(100) NOT NULL,
    UNIQUE (make_id, slug)
);

CREATE INDEX idx_car_models_make_id ON car_models (make_id);
