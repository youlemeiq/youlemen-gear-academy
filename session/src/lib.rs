#![no_std]
#![allow(warnings)]
use gstd::{exec, msg, prelude::*, ActorId, MessageId};
use session_game_io::*;
use wordle_game_io::*;

static mut GAMES: Option<State> = None;
static mut TARGET_PROGRAM_ID: ActorId = ActorId::zero();

fn is_word_lowercase(word: String) -> bool {
    if word.is_empty() {
        return false;
    }
    word.chars().all(|c| c.is_lowercase())
}

// Check if actorid exists and has a session, if so, the game exists, otherwise it does not exist
fn is_exist_game(user_id: &ActorId) -> bool {
    unsafe {
        if let Some(games) = GAMES.as_ref() {
            if games.get(user_id).is_some() {
                return true;
            }
        }
    }
    false
}

#[no_mangle]
extern "C" fn init() {
    // Receives and stores the Wordle program's address.
    let target_program_id = msg::load().expect("Unable to decode Init");

    unsafe {
        TARGET_PROGRAM_ID = target_program_id;
        GAMES = Some(State::new());
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: Action = msg::load().expect("Unable to decode `Action`");

    let user_id = msg::source();

    let games = unsafe { GAMES.as_mut().expect("The session is not initialized") };

    if !is_exist_game(&user_id) {
        let session: Session = Session {
            msg_ids: (MessageId::zero(), MessageId::zero()),
            session_status: SessionStatus::None,
            tries_number: 0,
        };

        games.insert(user_id, session);
    }

    match action {
        Action::StartGame => {
            let session = games.get_mut(&user_id).expect("Unable to decode Init");

            // let session_status = session.session_status.clone();
            if session.session_status == SessionStatus::None {
                unsafe {
                    let msg_id = msg::send(
                        TARGET_PROGRAM_ID,
                        WordleAction::StartGame { user: user_id },
                        0,
                    )
                    .expect("Error in sending a StartGame message");

                    session.msg_ids = (msg_id, msg::id());
                }

                msg::send_delayed(exec::program_id(), Action::CheckGameStatus, 0, 200)
                    .expect("Error in sending a CheckSelf Delayed message");

                exec::wait_for(3);
            } else if session.session_status == SessionStatus::GameStarted {
                session.session_status = SessionStatus::Waiting;
                exec::leave();
            } else if session.session_status == SessionStatus::GameOver(Outcome::Win)
                || session.session_status == SessionStatus::GameOver(Outcome::Lose)
            {
                unsafe {
                    let msg_id = msg::send(
                        TARGET_PROGRAM_ID,
                        WordleAction::StartGame { user: user_id },
                        0,
                    )
                    .expect("Error in sending a StartGame message");

                    let session = Session {
                        msg_ids: (msg_id, msg::id()),
                        session_status: SessionStatus::GameStarted,
                        tries_number: 0,
                    };

                    games.insert(user_id, session.clone());
                }

                exec::wait_for(3);
            }
        }

        Action::CheckWord(word) => {
            let session = games.get_mut(&user_id).expect("Unable to decode Init");
            if !is_exist_game(&user_id) {
                panic!("HANDLE: Action::CheckWord not_exist_game");
            }

            let session_status = session.session_status.clone();

            if session_status == SessionStatus::MessageSent {
                msg::reply(SessionStatus::MessageSent, 0)
                    .expect("Error in replying a MessageSent message");
                exec::leave();
            }

            if word.len() != 5 && !is_word_lowercase(word.clone()) {
                msg::reply(SessionStatus::InvalidWord, 0)
                    .expect("Error in replying a InvalidWord message");
                panic!("HANDLE: Action::CheckWord  invaild vord: {:?}", word);
            }

            if session_status == SessionStatus::Waiting {
                // let send_word = word.clone();
                unsafe {
                    let _msg_id = msg::send(
                        TARGET_PROGRAM_ID,
                        WordleAction::CheckWord {
                            user: user_id,
                            word,
                        },
                        0,
                    )
                    .expect("Error in sending a CheckWord message");
                }
                session.session_status = SessionStatus::MessageSent;
                exec::wait_for(20);
            }

            if session_status == SessionStatus::GameOver(Outcome::Win)
                || session_status == SessionStatus::GameOver(Outcome::Lose)
            {
                msg::reply(session_status, 0).expect("Error in replying a GameOver message");
            }

            exec::leave();
        }
        Action::CheckGameStatus => {
            let session = games.get_mut(&user_id).expect("Unable to decode Init");
            if session.session_status == SessionStatus::None && msg::source() == exec::program_id()
            {
                msg::send(user_id, SessionStatus::NoReplyReceived, 0)
                    .expect("Error in sending a message");
                session.session_status = SessionStatus::GameStarted;

                exec::wait_for(20);
            } else {
                let _ = msg::reply(session.session_status.clone(), 0);
            }

            exec::leave();
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    // Processes reply messages and updates the game status based on responses from the Wordle program.
    // Receives reply messages.
    // Utilizes msg::reply_to() to determine the message identifier, i.e., which message was replied to.

    // Processes and stores the result depending on the reply:
    // If a GameStarted response is received, it updates the game status to indicate that the game was successfully started.
    // If a WordChecked response is received, it saves the response, increments the number of tries, and checks if the word was guessed.
    // If the word has been guessed, it switches the game status to GameOver(Win).
    //If all attempts are used up and the word is not guessed, it switches the game status to GameOver(Lose).

    // Calls wake() with the identifier of the received message to acknowledge the response.

    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let reply_message = msg::load().expect("Unable to decode `Event`");
    let games = unsafe { GAMES.as_mut().expect("The session is not initialized") };

    match reply_message {
        WordleEvent::GameStarted { user } => {
            let session = games.get_mut(&user).expect("Failed to get session");

            if is_exist_game(&user) && reply_to == session.msg_ids.0 {
                session.session_status = SessionStatus::GameStarted;

                msg::send(user, SessionStatus::GameStarted, 0)
                    .expect("Error in sending a HANDLE_REPLY message");
            }

            exec::wake(session.msg_ids.1).expect("Failed to wake message");
        }

        WordleEvent::WordChecked {
            user,
            ref correct_positions,
            ref contained_in_word,
        } => {
            let correct_positions_c = correct_positions.clone();
            let contained_in_word_c = contained_in_word.clone();
            let session = games.get_mut(&user).expect("Failed to get session");

            msg::send(
                user,
                SessionStatus::WordChecked {
                    user,
                    correct_positions: correct_positions_c,
                    contained_in_word: contained_in_word_c,
                },
                0,
            )
            .expect("Error in sending a HANDLE_REPLY message");
            session.tries_number += 1;

            if correct_positions.len() == 5 && contained_in_word.is_empty() {
                session.session_status = SessionStatus::GameOver(Outcome::Win);
            } else if session.tries_number > 3 {
                session.session_status = SessionStatus::GameOver(Outcome::Lose);
            } else {
                session.session_status = SessionStatus::Waiting;
            }

            // WAKE for wait_for
            exec::wake(session.msg_ids.1).expect("Failed to wake message");
        }
    }
}

#[no_mangle]
pub extern "C" fn state() {
    // It is necessary to implement the state() function in order to get all the information about the game.
    let query = msg::load().expect("Failed to load query");
    let state = unsafe { GAMES.take().expect("Error in taking current state") };

    // Checks input data for validness
    let reply = match query {
        StateQuery::All => StateQueryReply::All(state.into_keys().collect()),
        StateQuery::Player(address) => {
            let session = state.get(&address).expect("Can't find this player");
            StateQueryReply::Game(session.clone())
        }
    };
    msg::reply(reply, 0).expect("Failed to reply state");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_word_lowercase() {
        assert!(is_word_lowercase("hello".to_string()));
        assert!(!is_word_lowercase("HELLO".to_string()));
        assert!(!is_word_lowercase("HeLLo".to_string()));
        assert!(!is_word_lowercase("".to_string()));
        assert!(!is_word_lowercase(" ".to_string()));
        assert!(!is_word_lowercase("12345".to_string()));
    }

    #[test]
    fn test_is_exist_game() {
        let user_id = ActorId::from([1; 32]);
        let mut games = State::new();
        let session = Session {
            msg_ids: (MessageId::zero(), MessageId::zero()),
            session_status: SessionStatus::None,
            tries_number: 0,
        };
        games.insert(user_id, session.clone());
        unsafe {
            GAMES = Some(games);
        }
        assert!(is_exist_game(&user_id));
        assert!(!is_exist_game(&ActorId::from([2; 32])));
    }
}
