use actix_web::{HttpResponse, web, HttpRequest};

use std::{env, sync::Arc};

use futures::future::join_all;

use crate::utils::{metadata::{Proposal, PROPOSALS, STATE, ACTIVE_NODES},ips::REMOTE_ADDRESS};

pub async fn secondry_index(proposal : web::Json<Proposal>, req : HttpRequest) -> HttpResponse {
    /*
     * insert the proposal locally.
     * */
    let new_proposal = Proposal {
        id : proposal.0.id,
        subject : proposal.0.subject.clone(),
        description : proposal.0.description.clone()
    };
    let mut proposals = PROPOSALS.lock();
    proposals.insert(proposal.0.id,new_proposal);
    drop(proposals);

    /*
     * Fetch the client-address from the header
     */
    let mut client_add : String = "".to_string();
    if let Some(caller) = req.headers().get("client-add") {
        if let Ok(caller_ip) = caller.to_str() {
            client_add = caller_ip.to_string();
        }
    }
    
    /*
     * find the ip of the current node; 
     * */
    let ip = env::var("IP").expect("Failed to Load the IP of the machine !!");
    let port = env::var("PORT").expect("Failed to fetch the Port !!");
    let this_ip = format!("{}:{}",ip,port);

    /*
    * find the active_nodes and there count;
    * */
    let mut futures = Vec::new();
    let remote = REMOTE_ADDRESS.lock();
    for add in &*remote {
        let _add = format!("{}",add);
        if _add == this_ip {
            continue;
        }
        let url = format!("http://{}/running", add.clone());
        let active_nodes_clone = Arc::clone(&ACTIVE_NODES);
        let future = async move {
            let _res = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(500)) // Set a timeout of 500 mili-seconds
                .build()
                .unwrap()
                .head(&url)
                .send()
                .await?;
            let mut active_nodes = active_nodes_clone.lock();
            if let Some(pending_active_nodes) =  active_nodes.get_mut(&proposal.0.id) {
                pending_active_nodes.push(_add.clone());
            }else {
                let mut new_active_nodes = Vec::new();
                new_active_nodes.push(_add.clone());
                active_nodes.insert(proposal.0.id, new_active_nodes);
            }
            drop(active_nodes);
            Result::<(), reqwest::Error>::Ok(())
        };
        futures.push(future);
    }
    drop(remote);
    join_all(futures).await;
    
    /*
    * now set the active_nodes, max_faulty_node and brodcast status locally.
    * */
    
    let _active_nodes = ACTIVE_NODES.lock();
    let _active_nodes_ = _active_nodes.get(&proposal.0.id).unwrap_or(&Vec::new()).clone();
    let n = _active_nodes_.len();
    let f = (n as i64)/3;
    println!("{:?}",_active_nodes_);
    drop(_active_nodes);
    let mut state = STATE.lock();
    state.insert(proposal.0.id, (n as i64 + 1, f, false,false));
    drop(state);
    //wait for 1.5 second to make sure all the nodes have the upper-bound on the number of faulty state.
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    /*
     * now it's time to send the proposal for the validation purpose.
     * */
    let _active_nodes_ = ACTIVE_NODES.lock();
    let _active_nodes = _active_nodes_.get(&proposal.0.id).unwrap_or(&Vec::new()).clone();
    drop(_active_nodes_);
    let mut futures = Vec::new(); 
    for _add in &*_active_nodes {
        let add = _add.clone();
        let url = format!("http://{}/signal", add.clone());
        let json_data = serde_json::to_string(&proposal).expect("Error while serializing");
        let _client_add = client_add.clone();
        let future = async move {
            let thread_id = std::thread::current().id();
            println!("Child thread ID: {:?}", thread_id);
            let _res = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20)) // Set a timeout of 20 seconds
                .build()
                .unwrap()
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header("client-add",_client_add.clone())
                .body(json_data)
                .send()
                .await?;
            Result::<(), reqwest::Error>::Ok(())
        };
        futures.push(future);
    }
    join_all(futures).await;

    /*
     * Everything is perfect/
     */
    HttpResponse::Ok().json("success")
}
