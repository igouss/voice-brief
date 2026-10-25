use serde::Deserialize;
use vb_contract::{Event, Placed, Violation};
use worker::{Context, Env, Request, Response, Result, event};

#[derive(Deserialize)]
struct Job {
    take: String,
    transcript: String,
    reply: String,
}

#[event(fetch)]
async fn fetch(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let job: Job = req.json().await?;
    match vb_contract::validate(&job.transcript, &job.reply) {
        Ok(placed) => {
            let placed: Placed = placed;
            let events: Vec<Event> = vb_contract::events(&job.take, placed);
            Response::from_json(&events)
        }
        Err(violations) => {
            let violations: Vec<Violation> = violations;
            Ok(Response::from_json(&violations)?.with_status(422))
        }
    }
}
