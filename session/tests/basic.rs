#![allow(warnings)]
use gstd::ActorId;
use gtest::{Log, Program, System};
use session_game_io::*;

const USER: u64 = 3;
const TARGET_PROGRAM_ADDRESS: u64 = 2;

#[test]
fn success_test() {
    // Create a new testing environment.
    let system = System::new();
    system.init_logger();
    // Get proxy program of the root crate with provided system.
    let proxy_program = Program::current(&system);
    // Get target program
    let target_program = Program::from_file(
        &system,
        "../wordle/target/wasm32-unknown-unknown/debug/wordle_game.wasm",
    );
    system.mint_to(USER, 9888989898989989);
    // The target program is initialized with an empty payload message
    let result = target_program.send_bytes(USER, []);
    system.run_next_block();
    let target_program_address: ActorId = TARGET_PROGRAM_ADDRESS.into();

    // The proxy program is initialized using target_program in the payload message
    let res = proxy_program.send(USER, target_program_address);
    system.run_next_block();
    // Send with the message we want to receive back
    let result = proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    println!("result:: {:?}", result);
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::GameStarted);
    println!("log:: {:?}", log);

    // User attempts to send another message to a proxy program while it is still processing the first message. It is expected that the proxy program will reply with the event `MessageAlreadySent`.
    let result = proxy_program.send(USER, Action::CheckWord("hhhhh".to_owned()));
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::WordChecked {
            user: USER.into(),
            correct_positions: vec![0],
            contained_in_word: vec![1, 2, 3, 4],
        });
    println!("log:: {:?}", log);
    proxy_program.send(USER, Action::CheckWord("hqqqq".to_owned()));
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::WordChecked {
            user: USER.into(),
            correct_positions: vec![0],
            contained_in_word: vec![],
        });
    println!("log:: {:?}", log);
    let result = proxy_program.send(USER, Action::CheckWord("qqqqq".to_owned()));
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::WordChecked {
            user: USER.into(),
            correct_positions: vec![],
            contained_in_word: vec![],
        });
    println!("log:: {:?}", log);
    let result = proxy_program.send(USER, Action::CheckWord("wwwww".to_owned()));
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::WordChecked {
            user: USER.into(),
            correct_positions: vec![],
            contained_in_word: vec![],
        });
    println!("log:: {:?}", log);
    let result = proxy_program.send(USER, Action::CheckGameStatus);
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::GameOver(Outcome::Lose));
    println!("log:: {:?}", log);
    // Restart this game, Only this game is GameOver.
    let result = proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::GameStarted);
    println!("log:: {:?}", log);
    let result = proxy_program.send(USER, Action::CheckGameStatus);
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::Waiting);
    println!("log:: {:?}", log);
    let result = proxy_program.send(USER, Action::CheckWord("house".to_owned()));
    system.run_next_block();
    let result = proxy_program.send(USER, Action::CheckWord("human".to_owned()));
    system.run_next_block();
    let result = proxy_program.send(USER, Action::CheckWord("horse".to_owned()));
    system.run_next_block();
    // Under probability conditions, the final game state is WIN.
    let result = proxy_program.send(USER, Action::CheckGameStatus);
    system.run_next_block();
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::GameOver(Outcome::Win));

    system.run_to_block(30);
    // if target_program handle() exist `exec::wait();`, `Event::NoReplyReceived` will be received.
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(SessionStatus::NoReplyReceived);
    println!("log:: {:?}", log);
}
