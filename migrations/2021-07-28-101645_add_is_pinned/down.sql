DELETE FROM log_types WHERE id in (9, 10);
ALTER TABLE topics DROP INDEX index_is_pinned;
ALTER TABLE topics DROP COLUMN is_pinned;
