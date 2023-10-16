use std::collections::BTreeMap;

use actix_web::{HttpResponse, web, HttpRequest};

use crate::utils::metadata::{VOTES, Vote, STATE, Reply};

use std::env;

pub async fn vote(vote : web::Json<Vote>, req : HttpRequest) -> HttpResponse {
    println!("VOTE FROM : {:?}",vote.0.ip);
    /*
     * Insert that vote. 
     */
    let mut votes = VOTES.lock();
    let id = vote.0.id;
    if let Some(all_votes) = votes.get_mut(&id) {
        all_votes.insert(vote.0.ip.clone(),vote.0.vote);
    }else {
        let mut new_map = BTreeMap::new();
        new_map.insert(vote.0.ip.clone(), vote.0.vote);
        votes.insert(id, new_map);
    }

    /*
     * Check if we get 2*f+1 votes or not.
     * if yes we will commit and send the result to the client. 
     */
    if let Some(all_votes) = votes.get(&id) {
        let mut n = 0;
        for (_,val) in all_votes {
            if *val == 1 {
                n += 1;
            }
        }
        drop(votes);
        let mut state = STATE.lock();
        if let Some(_state) = state.get_mut(&id) {
            if _state.3 == false && n as i64 >= 2*_state.1 + 1 {
                let f = _state.1;
                let total = _state.0;
                _state.3 = true;
                drop(state);
                /*
                * find the ip of the current machine.
                */
                let ip = env::var("IP").expect("Failed to Load the IP of the machine !!");
                let port = env::var("PORT").expect("Failed to fetch the Port !!");
                let this_ip = format!("{}:{}",ip,port);

                /*
                 * find the client address from the header. 
                 */
                let mut client_add : String = "".to_string();
                if let Some(caller) = req.headers().get("client-add") {
                    if let Ok(caller_ip) = caller.to_str() {
                        client_add = caller_ip.to_string();
                    }
                }

                /*
                 * Reply-body to the client.
                 */
                let _reply = Reply {
                    id : vote.0.id, from : this_ip.clone(), vote : 1, f, total
                };

                /*
                 * Replying to the client started.
                 * */
                println!("Replying to the client !!");
                let url = format!("http://{}/reply", client_add.clone());
                let json_data = serde_json::to_string(&_reply).expect("Error while serializing");
                let _client_add = client_add.clone();
                let future = async move {
                    let _res = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(7)) // Set a timeout of 7 seconds
                        .build()
                        .unwrap()
                        .post(&url)
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .header("client-add", _client_add)
                        .body(json_data)
                        .send()
                        .await?;
                    Result::<(), reqwest::Error>::Ok(())
                };
                let _res = future.await;
            }
        }
    }
    HttpResponse::Ok().json("success")
}