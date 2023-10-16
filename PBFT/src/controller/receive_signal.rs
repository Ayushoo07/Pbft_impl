use actix_web::{HttpResponse,web, HttpRequest};

use crate::{utils::metadata::{Proposal, PROPOSALS, FAVOR, STATE}, controller::brodcast_vote::brodcast_vote};

pub async fn receive_signal(proposal : web::Json<Proposal>, req : HttpRequest) -> HttpResponse {
    /*
     * find the client-address
     */
    let mut client_add : String = "".to_string();
    if let Some(caller) = req.headers().get("client-add") {
        if let Ok(caller_ip) = caller.to_str() {
            client_add = caller_ip.to_string();
        }
    }
    /*
     * check whether both the proposals are same or not. 
     * if yes increase the value of getting accepted by 1.
     */
    println!("RECEIVED A FAVOR");
    let proposals = PROPOSALS.lock();
    let received_proposal = proposals.get(&proposal.0.id).unwrap();
    if proposal.0 == *received_proposal {
        drop(proposals);
        let mut favor = FAVOR.lock();
        if let Some(val) = favor.get_mut(&proposal.0.id) {
            *val += 1;
        }else {
            favor.insert(proposal.0.id, 1);
        }
        drop(favor);
    }

    /*
     * now find the how many votes are in favor of the proposal.
     * if more than 2*f or equal are in favor brodcast the votes. 
     */
    let favor = FAVOR.lock();
    let total_favor = *favor.get(&proposal.0.id).unwrap_or_else(|| &0);
    drop(favor);
    let mut state = STATE.lock();
    let _state = state.get_mut(&proposal.0.id).unwrap();
    if _state.2 == false && 2*_state.1 <=  total_favor {
        /*
        * brodcast the vote to all the nodes
        * */
        _state.2 = true;
        drop(state);
        println!("BRODCAST START");
        let _res = brodcast_vote(proposal.0.id,client_add).await;
    }
    HttpResponse::Ok().json("success")
}