CREATE TABLE toss_payment_status(
    id BLOB NOT NULL PRIMARY KEY,
    user_id BLOB NOT NULL,
    temporary_reservation_id BLOB UNIQUE NOT NULL,
    booking_id BLOB UNIQUE,
    price INTEGER NOT NULL,
    payment_key TEXT,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    refund_price INTEGER,
    refunded_at DATETIME,

    FOREIGN KEY(user_id) REFERENCES user(id),
    FOREIGN KEY(temporary_reservation_id) REFERENCES adhoc_reservation(id),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_toss_payment_status_user_id ON toss_payment_status(user_id);
CREATE INDEX idx_toss_payment_status_temporary_reservation_id ON toss_payment_status(temporary_reservation_id);
CREATE INDEX idx_toss_payment_status_booking_id ON toss_payment_status(booking_id);
CREATE INDEX idx_toss_payment_status_created_at ON toss_payment_status(created_at);

ALTER TABLE adhoc_reservation DROP COLUMN temporary;
ALTER TABLE adhoc_reservation ADD COLUMN deleted_at DATETIME;
