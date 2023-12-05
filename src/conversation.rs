use rusqlite::Connection;

/// An OpenAI conversation
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub msec: f64,
}

impl Conversation {
    /// Write the conversation to the database
    pub fn write_to_database(&self, db: &Connection) -> rusqlite::Result<()> {
        db.execute(
            "INSERT INTO conversations (id, msec) VALUES (?1, ?2)",
            [&self.id, &self.msec.to_string()],
        )?;
        Ok(())
    }
}

/// A message exchange in the OpenAI conversation
pub struct Message {
    pub conversation_id: String,
    pub msec: f64,
    pub prompt: String,
    pub response: String,
}

impl Message {
    /// Write the message to the database
    pub fn write_to_database(&self, db: &Connection) -> rusqlite::Result<()> {
        db.execute(
            "INSERT INTO messages (conversation_id, msec, prompt, response) VALUES (?1, ?2, ?3, ?4)",
            [
                &self.conversation_id,
                &self.msec.to_string(),
                &self.prompt,
                &self.response,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    // return an in-memory database connection
    fn setup() -> Result<Connection, rusqlite::Error> {
        let db = Connection::open_in_memory()?;
        database::write_schema(&db, include_str!("schema.sql"))?;
        Ok(db)
    }

    #[test]
    fn test_conversation_write_to_database() {
        let db = setup().unwrap();
        let conversation = Conversation {
            id: "asst_7pF0CU0GNsBodf5XsVCcopFw".to_string(),
            messages: Vec::new(),
            msec: 0.0,
        };
        let message = Message {
            conversation_id: conversation.id.clone(),
            msec: 0.0,
            prompt: "What does Lorem Ipsum mean?".to_string(),
            response: "It doesn't mean anything, you idiot!".to_string(),
        };
        conversation.write_to_database(&db).unwrap();
        message.write_to_database(&db).unwrap();

        // verify conversation data
        let mut stmt = db.prepare("SELECT id, msec FROM conversations").unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(Conversation {
                    id: row.get(0).unwrap(),
                    messages: Vec::new(),
                    msec: row.get(1).unwrap(),
                })
            })
            .unwrap();
        for row in rows {
            let c = row.unwrap();
            assert_eq!(c.id, conversation.id);
            assert_eq!(c.msec, conversation.msec);
        }

        // verify message data
        let mut stmt = db
            .prepare("SELECT conversation_id, msec, prompt, response FROM messages")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0).unwrap(),
                    row.get::<_, f64>(1).unwrap(),
                    row.get::<_, String>(2).unwrap(),
                    row.get::<_, String>(3).unwrap(),
                ))
            })
            .unwrap();
        for row in rows {
            let row = row.unwrap();
            assert_eq!(row.0, message.conversation_id);
            assert_eq!(row.1, message.msec);
            assert_eq!(row.2, message.prompt);
            assert_eq!(row.3, message.response);
        }
    }
}
