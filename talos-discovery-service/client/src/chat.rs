pub(crate) struct Chat {
    pub(crate) chatroom: String,
    username: String,
    password: Option<String>,
}

impl Chat {
    pub fn new(chatroom: String, username: String, password: Option<String>) -> Self {
        Self {
            chatroom,
            username,
            password,
        }
    }

    pub async fn execute(&self) {}
}
