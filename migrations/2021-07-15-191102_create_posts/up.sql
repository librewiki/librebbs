-- Your SQL goes here
CREATE TABLE posts (
    id INT PRIMARY KEY AUTO_INCREMENT,
    board_id INT NOT NULL,
    title VARCHAR(500) NOT NULL,
    content MEDIUMTEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp,
    FOREIGN KEY (board_id) REFERENCES boards(id) ON UPDATE CASCADE,
    INDEX (created_at),
    INDEX (updated_at)
);
