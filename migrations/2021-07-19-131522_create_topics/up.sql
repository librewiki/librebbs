CREATE TABLE topics (
    id INT PRIMARY KEY AUTO_INCREMENT,
    board_id INT NOT NULL,
    title VARCHAR(500) NOT NULL,
    author_id INT NULL,
    author_name VARCHAR(100) NULL,
    author_ip BINARY(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp,
    FOREIGN KEY (board_id) REFERENCES boards(id) ON UPDATE CASCADE,
    INDEX (author_id),
    INDEX (author_name),
    INDEX (author_ip),
    INDEX (created_at),
    INDEX (updated_at)
);
