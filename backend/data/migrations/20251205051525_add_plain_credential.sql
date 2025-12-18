CREATE TABLE user_plain_credential(
    user_id BLOB NOT NULL PRIMARY KEY,
    handle VARCHAR(40) NOT NULL UNIQUE,
    argon2_password TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES user(id)
);

ALTER TABLE user ADD COLUMN use_pg_payment BOOLEAN NOT NULL DEFAULT false;
