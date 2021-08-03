ALTER TABLE topics ADD COLUMN comment_count INT NOT NULL DEFAULT 0 AFTER is_pinned;
ALTER TABLE topics ADD INDEX index_comment_count (comment_count);

UPDATE topics
    SET comment_count = (SELECT COUNT(c.id) FROM comments c WHERE c.topic_id = topics.id),
        updated_at = topics.updated_at;

CREATE TRIGGER update_comment_count_on_insert
AFTER INSERT ON comments FOR EACH ROW
    UPDATE topics
    SET comment_count = (SELECT COUNT(c.id) FROM comments c WHERE c.topic_id = topics.id)
    WHERE topics.id = new.topic_id;

CREATE TRIGGER update_comment_count_on_delete
AFTER DELETE ON comments FOR EACH ROW
    UPDATE topics
    SET comment_count = (SELECT COUNT(c.id) FROM comments c WHERE c.topic_id = topics.id)
    WHERE topics.id = old.topic_id;

CREATE TRIGGER update_comment_count_on_update
AFTER UPDATE ON comments FOR EACH ROW
    UPDATE topics
    SET comment_count = (SELECT COUNT(c.id) FROM comments c WHERE c.topic_id = topics.id)
    WHERE topics.id = new.topic_id or topics.id = old.topic_id;
