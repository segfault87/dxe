PRAGMA foreign_keys = OFF;

CREATE TABLE adhoc_reservation(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    unit_id VARCHAR(20) NOT NULL,
    holder_id BLOB NOT NULL,
    customer_id BLOB NOT NULL,
    time_from DATETIME NOT NULL,
    time_to DATETIME NOT NULL,
    temporary BOOLEAN NOT NULL,
    remark TEXT,
    FOREIGN KEY(unit_id) REFERENCES unit(id),
    FOREIGN KEY(holder_id) REFERENCES user(id),
    FOREIGN KEY(customer_id) REFERENCES identity(id)
);
CREATE INDEX idx_adhoc_reservation_unit_id ON adhoc_reservation(unit_id);
CREATE INDEX idx_adhoc_reservation_holder_id ON adhoc_reservation(holder_id);
CREATE INDEX idx_adhoc_reservation_customer_id ON adhoc_reservation(customer_id);
CREATE INDEX idx_adhoc_reservation_time_from ON adhoc_reservation(time_from);
CREATE INDEX idx_adhoc_reservation_time_to ON adhoc_reservation(time_to);

INSERT INTO adhoc_reservation
    (id, unit_id, holder_id, customer_id, time_from, time_to, temporary, remark)
SELECT id, unit_id, holder_id, holder_id, time_from, time_to, temporary, remark
FROM reservation;

DROP TABLE reservation;

PRAGMA foreign_keys = ON;
