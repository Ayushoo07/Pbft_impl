use actix_web::{web, HttpResponse, HttpRequest};

use crate::utils::metadata::Proposal;

use crate::controller::send_proposal::send_proposal;

pub async fn index(proposal : web::Json<Proposal>, req : HttpRequest) -> HttpResponse {
    let mut client_add : String = "".to_string();
    if let Some(caller) = req.headers().get("client-add") {
        if let Ok(caller_ip) = caller.to_str() {
            client_add = caller_ip.to_string();
        }
    }

    /*
     * sending the proposal to all the nodes.
     * */
    println!("Proposal came !!");
    let res = send_proposal(&proposal,client_add.clone()).await;
    match res {
        Ok(()) => {
            HttpResponse::Ok().json("success")  
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(format!("ErrorMessage : {}",err))
        }
    }
}
