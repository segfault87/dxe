CREATE TABLE adhoc_payment(
    id BLOB NOT NULL PRIMARY KEY,
    issuer_id BLOB NOT NULL,
    recipient_id BLOB NOT NULL,
    price INTEGER NOT NULL,
    note TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    paid_at DATETIME,
    canceled_at DATETIME,
    FOREIGN KEY(id) REFERENCES product(id),
    FOREIGN KEY(issuer_id) REFERENCES user(id),
    FOREIGN KEY(recipient_id) REFERENCES identity(id)
);
CREATE INDEX idx_adhoc_payment_recipient_id ON adhoc_payment(recipient_id);
CREATE INDEX idx_adhoc_payment_created_at ON adhoc_payment(created_at);
