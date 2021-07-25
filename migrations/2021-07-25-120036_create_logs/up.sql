CREATE TABLE log_types (
    id INT PRIMARY KEY,
    name VARCHAR(100) NULL
);

INSERT INTO log_types (id, name) VALUES (1, "CLOSE_TOPIC"),
                                        (2, "UNCLOSE_TOPIC"),
                                        (3, "HIDE_TOPIC"),
                                        (4, "UNHIDE_TOPIC"),
                                        (5, "SUSPEND_TOPIC"),
                                        (6, "UNSUSPEND_TOPIC"),
                                        (7, "HIDE_COMMENT"),
                                        (8, "UNHIDE_COMMENT");

CREATE TABLE logs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    log_type_id INT NOT NULL,
    content VARCHAR(2000) NOT NULL,
    user_id INT NULL,
    user_name VARCHAR(100) NULL,
    user_ip BINARY(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    FOREIGN KEY (log_type_id) REFERENCES log_types(id) ON UPDATE CASCADE,
    INDEX (user_id),
    INDEX (user_name),
    INDEX (user_ip),
    INDEX (created_at)
);
