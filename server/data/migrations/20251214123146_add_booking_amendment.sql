ALTER TABLE user DROP COLUMN use_pg_payment;

CREATE TABLE product(
    id BLOB NOT NULL PRIMARY KEY,
    discriminator VARCHAR(30) NOT NULL
);

ALTER TABLE booking RENAME TO old_booking;

DROP INDEX idx_booking_holder_id;
DROP INDEX idx_booking_customer_id;
DROP INDEX idx_booking_time_from;
DROP INDEX idx_booking_time_to;
DROP INDEX idx_booking_created_at;

CREATE TABLE booking(
    id BLOB NOT NULL PRIMARY KEY,
    unit_id VARCHAR(20) NOT NULL,
    holder_id BLOB NOT NULL,
    customer_id BLOB NOT NULL,
    time_from DATETIME NOT NULL,
    time_to DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    canceled_at DATETIME,
    FOREIGN KEY(id) REFERENCES product(id),
    FOREIGN KEY(unit_id) REFERENCES unit(id),
    FOREIGN KEY(holder_id) REFERENCES user(id),
    FOREIGN KEY(customer_id) REFERENCES identity(id)
);
CREATE INDEX idx_booking_holder_id ON booking(holder_id);
CREATE INDEX idx_booking_customer_id ON booking(customer_id);
CREATE INDEX idx_booking_time_from ON booking(time_from);
CREATE INDEX idx_booking_time_to ON booking(time_to);
CREATE INDEX idx_booking_created_at ON booking(created_at);

INSERT INTO product(id, discriminator) SELECT id, 'booking' FROM old_booking;
INSERT INTO booking SELECT * from old_booking;

ALTER TABLE audio_recording RENAME TO old_audio_recording;
CREATE TABLE audio_recording(
    booking_id BLOB NOT NULL PRIMARY KEY,
    url TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    expires_in DATETIME,
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
INSERT INTO audio_recording SELECT * FROM old_audio_recording;
DROP TABLE old_audio_recording;

ALTER TABLE telemetry_file RENAME TO old_telemetry_file;
DROP INDEX idx_telemetry_file_booking_id;
CREATE TABLE telemetry_file(
    booking_id BLOB NOT NULL,
    type VARCHAR(30) NOT NULL,
    file_name TEXT NOT NULL,
    uploaded_at DATETIME NOT NULL,
    PRIMARY KEY(booking_id, type),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_telemetry_file_booking_id ON telemetry_file(booking_id);
INSERT INTO telemetry_file SELECT * from old_telemetry_file;
DROP TABLE old_telemetry_file;

CREATE TABLE booking_amendment(
    id BLOB NOT NULL PRIMARY KEY,
    booking_id BLOB NOT NULL,
    original_time_from DATETIME NOT NULL,
    original_time_to DATETIME NOT NULL,
    desired_time_from DATETIME NOT NULL,
    desired_time_to DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    canceled_at DATETIME,
    FOREIGN KEY(id) REFERENCES product(id),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_booking_amendment_booking_id ON booking_amendment(booking_id);
CREATE INDEX idx_booking_amendment_created_at ON booking_amendment(created_at);

CREATE TABLE cash_transaction(
    product_id BLOB NOT NULL PRIMARY KEY,
    depositor_name VARCHAR(40) NOT NULL,
    price INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    refund_price INTEGER,
    refunded_at DATETIME,
    refund_account VARCHAR(80),
    FOREIGN KEY(product_id) REFERENCES product(id)
);

INSERT INTO cash_transaction(
    product_id, depositor_name, price, created_at, confirmed_at, refund_price, refunded_at, refund_account
)
SELECT
    booking_id, depositor_name, price, created_at, confirmed_at, refund_price, refunded_at, refund_account
FROM cash_payment_status;

CREATE TABLE toss_payments_transaction(
    id BLOB NOT NULL PRIMARY KEY,
    user_id BLOB NOT NULL,
    temporary_reservation_id BLOB UNIQUE,
    product_id BLOB UNIQUE,
    price INTEGER NOT NULL,
    payment_key TEXT,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    refund_price INTEGER,
    refunded_at DATETIME,

    FOREIGN KEY(user_id) REFERENCES user(id),
    FOREIGN KEY(temporary_reservation_id) REFERENCES adhoc_reservation(id),
    FOREIGN KEY(product_id) REFERENCES product(id)
);
CREATE INDEX idx_toss_payments_transaction_user_id ON toss_payments_transaction(user_id);
CREATE INDEX idx_toss_payments_transaction_temporary_reservation_id ON toss_payments_transaction(temporary_reservation_id);
CREATE INDEX idx_toss_payments_transaction_product_id ON toss_payments_transaction(product_id);
CREATE INDEX idx_toss_payments_transaction_created_at ON toss_payments_transaction(created_at);

INSERT INTO toss_payments_transaction(
    id, user_id, temporary_reservation_id, product_id, price, payment_key, created_at, confirmed_at, refund_price, refunded_at
)
SELECT
    id, user_id, temporary_reservation_id, booking_id, price, payment_key, created_at, confirmed_at, refund_price, refunded_at
FROM toss_payment_status;

DROP TABLE cash_payment_status;
DROP TABLE toss_payment_status;
DROP TABLE old_booking;
DROP TABLE booking_change_history;
