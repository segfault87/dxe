CREATE TABLE mixer_config(
    identity_id BLOB NOT NULL PRIMARY KEY,
    unit_id VARCHAR(20) NOT NULL,
    data JSONB NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY(identity_id) REFERENCES identity(id),
    UNIQUE (identity_id, unit_id)
);
CREATE INDEX idx_mixer_config_created_at ON mixer_config(created_at);
CREATE INDEX idx_mixer_config_unit_id ON mixer_config(unit_id);
