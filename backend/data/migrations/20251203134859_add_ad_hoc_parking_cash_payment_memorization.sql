CREATE TABLE adhoc_parking(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    space_id VARCHAR(20) NOT NULL,
    time_from DATETIME NOT NULL,
    time_to DATETIME NOT NULL,
    license_plate_number VARCHAR(30) NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY(space_id) REFERENCES space(id)
);
CREATE INDEX idx_adhoc_parking_space_id ON adhoc_parking(space_id);
CREATE INDEX idx_adhoc_parking_time_from ON adhoc_parking(time_from);
CREATE INDEX idx_adhoc_parking_time_to ON adhoc_parking(time_to);

CREATE TABLE user_cash_payment_information(
    user_id BLOB NOT NULL PRIMARY KEY,
    depositor_name VARCHAR(40),
    refund_account VARCHAR(80),
    FOREIGN KEY(user_id) REFERENCES user(id)
);
