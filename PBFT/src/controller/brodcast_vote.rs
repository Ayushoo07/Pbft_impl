use crate::utils::metadata::{ACTIVE_NODES, Vote, VOTES};

use std::env;

use std::collections::BTreeMap;

use super::vote_result::validation;

use futures::future::join_all;

pub async fn brodcast_vote(id : i64, client_add : String) -> Result<(),reqwest::Error> {
    /*
     * take all the active nodes to call on.
     */
    let active_nodes = ACTIVE_NODES.lock();
    let _active_nodes = active_nodes.get(&id).unwrap_or(&Vec::new()).clone();
    drop(active_nodes);

    /*
     * find the ip of the current machine.
     */
    let ip = env::var("IP").expect("Failed to Load the IP of the machine !!");
    let port = env::var("PORT").expect("Failed to fetch the Port !!");
    let this_ip = format!("{}:{}",ip,port);

    /*
     * vote 
     */
    let vote = Vote {
        id,
        ip : this_ip,
        vote : validation()
    };
    /*
     * Give your own vote. 
     */
    let mut votes = VOTES.lock();
    if let Some(all_votes) = votes.get_mut(&id) {
        all_votes.insert(vote.ip.clone(),vote.vote);
    }else {
        let mut new_map = BTreeMap::new();
        new_map.insert(vote.ip.clone(), vote.vote);
        votes.insert(id, new_map);
    }
    drop(votes);
    /*
     * Brodcating the vote to all the ips. 
     */
    let mut futures = Vec::new(); 
    for add in _active_nodes {
        let url = format!("http://{}/vote", add.clone());
        let json_data = serde_json::to_string(&vote).expect("Error while serializing");
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
        futures.push(future);
    }
    join_all(futures).await;
    Ok(())
}