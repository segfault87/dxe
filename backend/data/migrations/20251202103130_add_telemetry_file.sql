CREATE TABLE telemetry_file(
    booking_id BLOB NOT NULL,
    type VARCHAR(30) NOT NULL,
    file_name TEXT NOT NULL,
    uploaded_at DATETIME NOT NULL,
    PRIMARY KEY(booking_id, type),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_telemetry_file_booking_id ON telemetry_file(booking_id);
