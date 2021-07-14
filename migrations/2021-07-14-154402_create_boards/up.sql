-- Your SQL goes here
CREATE TABLE boards (
    id INT PRIMARY KEY AUTO_INCREMENT,
    display_name VARCHAR(255) NOT NULL,
    name VARCHAR(255) UNIQUE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO boards (display_name, name) VALUES ('위키방', 'wiki');
