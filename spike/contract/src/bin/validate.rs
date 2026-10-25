use std::{env, fs, process::ExitCode};

use vb_contract::{Event, Placed, Violation};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let [_, take, transcript, reply] = args.as_slice() else {
        eprintln!("usage: validate <take-id> <transcript-file> <reply-file>");
        return ExitCode::from(2);
    };
    let transcript: String = fs::read_to_string(transcript).expect("read transcript");
    let reply: String = fs::read_to_string(reply).expect("read reply");
    match vb_contract::validate(&transcript, &reply) {
        Ok(placed) => {
            let placed: Placed = placed;
            let events: Vec<Event> = vb_contract::events(take, placed);
            println!("{}", serde_json::to_string_pretty(&events).expect("serialize"));
            ExitCode::SUCCESS
        }
        Err(violations) => {
            let violations: Vec<Violation> = violations;
            for v in violations {
                println!("{v:?}");
            }
            ExitCode::FAILURE
        }
    }
}
