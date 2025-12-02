CREATE TABLE identity(
    id BLOB NOT NULL PRIMARY KEY,
    discriminator VARCHAR(5) NOT NULL
);

CREATE TABLE space(
    id VARCHAR(20) NOT NULL PRIMARY KEY,
    enabled BOOLEAN NOT NULL
);
INSERT INTO space(id, enabled) VALUES('default', 1);

CREATE TABLE unit(
    id VARCHAR(20) NOT NULL PRIMARY KEY,
    space_id VARCHAR(20) NOT NULL,
    enabled BOOLEAN NOT NULL,
    FOREIGN KEY(space_id) REFERENCES space(id)
);
INSERT INTO unit(id, space_id, enabled) VALUES('default', 'default', 1);

CREATE TABLE user(
    id BLOB NOT NULL PRIMARY KEY,
    provider VARCHAR(20) NOT NULL,
    foreign_id VARCHAR(30) NOT NULL,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    deactivated_at DATETIME,
    license_plate_number VARCHAR(30),
    FOREIGN KEY(id) REFERENCES identity(id),
    UNIQUE(provider, foreign_id)
);

CREATE TABLE "administrator"(
    id BLOB NOT NULL PRIMARY KEY,
    FOREIGN KEY(id) REFERENCES user(id)
);

CREATE TABLE "group"(
    id BLOB NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    owner_id BLOB NOT NULL,
    created_at DATETIME NOT NULL,
    deleted_at DATETIME,
    is_open BOOLEAN NOT NULL,
    FOREIGN KEY(owner_id) REFERENCES user(id)
);

CREATE TABLE group_association(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    group_id BLOB NOT NULL,
    user_id BLOB NOT NULL,
    joined_at DATETIME NOT NULL,
    FOREIGN KEY(group_id) REFERENCES "group"(id),
    FOREIGN KEY(user_id) REFERENCES user(id),
    UNIQUE(group_id, user_id)
);
CREATE INDEX idx_group_association_group_id ON group_association(group_id);

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
    FOREIGN KEY(unit_id) REFERENCES unit(id),
    FOREIGN KEY(holder_id) REFERENCES user(id),
    FOREIGN KEY(customer_id) REFERENCES identity(id)
);
CREATE INDEX idx_booking_holder_id ON booking(holder_id);
CREATE INDEX idx_booking_customer_id ON booking(customer_id);
CREATE INDEX idx_booking_time_from ON booking(time_from);
CREATE INDEX idx_booking_time_to ON booking(time_to);
CREATE INDEX idx_booking_created_at ON booking(created_at);

CREATE TABLE reservation(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    unit_id VARCHAR(20) NOT NULL,
    holder_id BLOB NOT NULL,
    time_from DATETIME NOT NULL,
    time_to DATETIME NOT NULL,
    temporary BOOLEAN NOT NULL,
    remark TEXT,
    FOREIGN KEY(unit_id) REFERENCES unit(id),
    FOREIGN KEY(holder_id) REFERENCES user(id)
);
CREATE INDEX idx_reservation_unit_id ON reservation(unit_id);
CREATE INDEX idx_reservation_holder_id ON reservation(holder_id);
CREATE INDEX idx_reservation_time_from ON reservation(time_from);
CREATE INDEX idx_reservation_time_to ON reservation(time_to);

CREATE TABLE booking_change_history(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    booking_id BLOB NOT NULL,
    created_at DATETIME NOT NULL,
    new_customer_id BLOB,
    new_time_from DATETIME,
    new_time_to DATETIME,
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_booking_change_history ON booking_change_history(booking_id);

CREATE TABLE activity(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id BLOB NOT NULL,
    booking_id BLOB,
    event_name VARCHAR(30),
    level VARCHAR(10) NOT NULL,
    created_at DATETIME NOT NULL,
    payload TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES user(id),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
CREATE INDEX idx_activity_user_id ON activity(user_id);
CREATE INDEX idx_activity_booking_id ON activity(booking_id);

CREATE TABLE cash_payment_status(
    booking_id BLOB NOT NULL PRIMARY KEY,
    depositor_name VARCHAR(40) NOT NULL,
    price INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    confirmed_at DATETIME,
    refund_price INTEGER,
    refunded_at DATETIME,
    refund_account VARCHAR(80),
    FOREIGN KEY(booking_id) REFERENCES booking(id)
);
