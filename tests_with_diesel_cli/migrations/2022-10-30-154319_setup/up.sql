CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE simple (
  id SERIAL PRIMARY KEY,
  some_value my_enum NOT NULL
);
