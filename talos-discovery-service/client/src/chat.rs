pub(crate) struct Chat {
    pub(crate) chatroom: String,
    _username: String,
    _password: Option<String>,
}

impl Chat {
    pub fn new(chatroom: String, username: String, password: Option<String>) -> Self {
        Self {
            chatroom,
            _username: username,
            _password: password,
        }
    }

    pub async fn execute(&self) {}
}
