use crate::model::ModuleState;
use actix::Message;

#[derive(Message)]
#[rtype("usize")]
pub struct CreateModule {
    pub priority: u8,
}

#[derive(Message)]
#[rtype("()")]
pub struct UpdateModule {
    pub id: usize,
    pub state: ModuleState,
}

#[derive(Message)]
#[rtype("()")]
pub struct RemoveModule {
    pub id: usize,
}

#[derive(Message, Debug)]
#[rtype("()")]
pub struct Update(pub String);
