CREATE TABLE audio_recording(
    booking_id BLOB NOT NULL PRIMARY KEY,
    url TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    expires_in DATETIME,
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
