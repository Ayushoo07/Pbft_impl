use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    pub static ref REMOTE_ADDRESS : Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn add_node(node : String ) {
    let mut remote_add = REMOTE_ADDRESS.lock();
    remote_add.push(node);
}