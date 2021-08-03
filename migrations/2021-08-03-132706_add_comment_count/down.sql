DROP TRIGGER update_comment_count_on_update;
DROP TRIGGER update_comment_count_on_delete;
DROP TRIGGER update_comment_count_on_insert;
ALTER TABLE topics DROP INDEX index_comment_count;
ALTER TABLE topics DROP COLUMN comment_count;
