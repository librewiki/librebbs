CREATE TABLE comments (
    id INT PRIMARY KEY AUTO_INCREMENT,
    topic_id INT NOT NULL,
    content MEDIUMTEXT NOT NULL,
    author_id INT NULL,
    author_name VARCHAR(100) NULL,
    author_ip VARBINARY(16) NOT NULL,
    is_hidden BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp,
    FOREIGN KEY (topic_id) REFERENCES topics(id) ON UPDATE CASCADE,
    INDEX (author_id),
    INDEX (author_name),
    INDEX (author_ip),
    INDEX (created_at),
    INDEX (updated_at)
);
