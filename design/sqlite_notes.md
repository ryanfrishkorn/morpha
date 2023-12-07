# SQLite Notes

### FTS5 Extension

Create full text search virtual table

```
CREATE VIRTUAL TABLE messages_search USING fts5(prompt, response, conversation_id);
INSERT INTO messages_search SELECT prompt, response, conversation_id FROM messages;
```

**Search and Highlight**

```
SELECT conversation_id, highlight(message_search, 1, '<b>', '</b>') FROM message_search WHERE response MATCH 'Ohaguro';
```

```
+-------------------------------+--------------------------------------------------------------+
|        conversation_id        |         highlight(message_search, 1, '<b>', '</b>')          |
+-------------------------------+--------------------------------------------------------------+
| asst_RomomWkdvxL2WJBUKTR70rrj | Ah, the Japanese practice of dyeing teeth black, known as "< |
|                               | b>ohaguro</b>", was indeed a common custom during feudal tim |
|                               | es. It was considered a mark of beauty and a symbol of matur |
|                               | ity, particularly among married women. The method involved u |
|                               | sing a mixture of iron filings, vinegar, and tannin-rich ing |
|                               | redients to create a black dye. This mixture was applied to  |
|                               | the teeth, resulting in a lustrous black appearance. The pra |
|                               | ctice was eventually phased out during the Meiji era as Japa |
|                               | n modernized and adopted Western customs. It is a fascinatin |
|                               | g aspect of historical aesthetics and cultural traditions.   |
+-------------------------------+--------------------------------------------------------------+
```
