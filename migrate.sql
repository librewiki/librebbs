-- 129 == 위키방 (1)
-- 131 == 익명방 (3)
-- 131207 == 조합 (4)
-- 176713 == 자유게시판 (2)
-- 208869 == 이슈트래커 (5)
USE librebbs;

INSERT INTO librebbs.topics
           (id, board_id, title, author_id, author_name, author_ip, is_closed, is_suspended, is_hidden, created_at, updated_at)
SELECT document_srl, 1, title, NULL, nick_name, INET6_ATON(ipaddress), TRUE, FALSE, FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE module_srl = 129;

INSERT INTO librebbs.topics
           (id, board_id, title, author_id, author_name, author_ip, is_closed, is_suspended, is_hidden, created_at, updated_at)
SELECT document_srl, 3, title, NULL, nick_name, INET6_ATON(ipaddress), TRUE, FALSE, FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE module_srl = 131;

INSERT INTO librebbs.topics
           (id, board_id, title, author_id, author_name, author_ip, is_closed, is_suspended, is_hidden, created_at, updated_at)
SELECT document_srl, 4, title, NULL, nick_name, INET6_ATON(ipaddress), TRUE, FALSE, FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE module_srl = 131207;

INSERT INTO librebbs.topics
           (id, board_id, title, author_id, author_name, author_ip, is_closed, is_suspended, is_hidden, created_at, updated_at)
SELECT document_srl, 2, title, NULL, nick_name, INET6_ATON(ipaddress), TRUE, FALSE, FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE module_srl = 176713;

INSERT INTO librebbs.topics
           (id, board_id, title, author_id, author_name, author_ip, is_closed, is_suspended, is_hidden, created_at, updated_at)
SELECT document_srl, 5, title, NULL, nick_name, INET6_ATON(ipaddress), TRUE, FALSE, FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE module_srl = 208869;

INSERT INTO librebbs.comments
    (topic_id, content, author_id, author_name, author_ip, is_hidden, created_at, updated_at)
SELECT document_srl, content, NULL, nick_name, INET6_ATON(ipaddress), FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_documents
    WHERE EXISTS (SELECT * FROM librebbs.topics topics WHERE topics.id = document_srl);

INSERT INTO librebbs.comments
    (topic_id, content, author_id, author_name, author_ip, is_hidden, created_at, updated_at)
SELECT document_srl, content, NULL, nick_name, INET6_ATON(ipaddress), FALSE, STR_TO_DATE(regdate, "%Y%m%d%H%i%s"), STR_TO_DATE(last_update, "%Y%m%d%H%i%s")
    FROM oldbbs.libre_comments
    WHERE EXISTS (SELECT * FROM librebbs.topics topics WHERE topics.id = document_srl)
    ORDER BY comment_srl asc;

UPDATE topics
    SET comment_count = (SELECT COUNT(c.id) FROM comments c WHERE c.topic_id = topics.id),
        updated_at = topics.updated_at;
