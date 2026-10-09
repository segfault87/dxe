CREATE TABLE booking_amendment_attachment_mapping(
    booking_amendment_id BLOB NOT NULL,
    attachment_id BLOB NOT NULL,
    PRIMARY KEY(booking_amendment_id, attachment_id),
    FOREIGN KEY(booking_amendment_id) REFERENCES booking_amendment(id),
    FOREIGN KEY(attachment_id) REFERENCES booking_attachment(id)
);
CREATE INDEX idx_booking_amendment_attachment_mapping_attachment_id ON booking_amendment_attachment_mapping(attachment_id);
