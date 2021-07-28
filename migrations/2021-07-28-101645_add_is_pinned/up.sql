ALTER TABLE topics ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT false AFTER is_hidden;
ALTER TABLE topics ADD INDEX index_is_pinned (is_pinned);
INSERT INTO log_types (id, name) VALUES (9, "PIN_TOPIC"),
                                        (10, "UNPIN_TOPIC");
