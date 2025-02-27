#[no_std]
use gmeta::metawasm;
use gstd::{prelude::*, ActorId, MessageId, Vec};
use session_game_io::{SessionStatus, State as GameIOState};

#[metawasm]
pub mod metafns {
    pub type State = GameIOState;

    pub fn all_player_address(state: State) -> Vec<ActorId> {
        state.into_keys().collect()
    }

    pub fn get_session_status(state: State, user_id: ActorId) -> SessionStatus {
        let session = state.get(&user_id).expect("Can't find this player");
        session.session_status
    }

    pub fn get_tries_number(state: State, user_id: ActorId) -> u8 {
        let session = state.get(&user_id).expect("Can't find this player");
        session.tries_number
    }

    pub fn get_sent_message_id(state: State, user_id: ActorId) -> MessageId {
        let session = state.get(&user_id).expect("Can't find this player");
        session.msg_ids.0
    }

    pub fn get_original_message_id(state: State, user_id: ActorId) -> MessageId {
        let session = state.get(&user_id).expect("Can't find this player");
        session.msg_ids.1
    }

}
