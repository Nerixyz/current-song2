use crate::model::{ModuleState, PlayInfo};
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

impl UpdateModule {
    pub fn paused(id: usize) -> Self {
        Self {
            id,
            state: ModuleState::Paused,
        }
    }

    pub fn playing(id: usize, play_info: PlayInfo) -> Self {
        Self {
            id,
            state: ModuleState::Playing(play_info),
        }
    }
}

#[derive(Message)]
#[rtype("()")]
pub struct RemoveModule {
    pub id: usize,
}
