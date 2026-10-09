CREATE TABLE booking_attachment(
    id BLOB NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    category VARCHAR(40) NOT NULL,
    price INTEGER NOT NULL,
    disabled_at DATETIME
);
CREATE INDEX idx_booking_attachment_category ON booking_attachment(category);

CREATE TABLE booking_attachment_mapping(
    booking_id BLOB NOT NULL,
    attachment_id BLOB NOT NULL,
    PRIMARY KEY(booking_id, attachment_id),
    FOREIGN KEY(booking_id) REFERENCES booking(id),
    FOREIGN KEY(attachment_id) REFERENCES booking_attachment(id)
);
CREATE INDEX idx_booking_attachment_mapping_attachment_id ON booking_attachment_mapping(attachment_id);

CREATE TABLE adhoc_reservation_attachment_mapping(
    adhoc_reservation_id INTEGER NOT NULL,
    attachment_id BLOB NOT NULL,
    PRIMARY KEY(adhoc_reservation_id, attachment_id),
    FOREIGN KEY(adhoc_reservation_id) REFERENCES adhoc_reservation(id),
    FOREIGN KEY(attachment_id) REFERENCES booking_attachment(id)
);
CREATE INDEX idx_adhoc_reservation_attachment_mapping_attachment_id ON adhoc_reservation_attachment_mapping(attachment_id);
