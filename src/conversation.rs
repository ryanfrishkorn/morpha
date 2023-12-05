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
