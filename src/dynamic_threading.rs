use super::{
    HTTPClient,
    log,
};

use std::{
    collections::HashMap,
    thread::{
        Builder,
        JoinHandle
    },
    sync::{
        Arc,
        Mutex,
    },
};

pub struct DynamicThreading{
    counter:usize,
    pool:Arc<Mutex<HashMap<usize,JoinHandle<()>>>>,
    thread_stack_memory:usize,
}

impl DynamicThreading{
    pub fn new(limit:usize,thread_stack_memory:usize)->DynamicThreading{
        Self{
            counter:0,
            pool:Arc::new(Mutex::new(HashMap::with_capacity(limit))),
            thread_stack_memory,
        }
    }

    pub fn handle_client(
        &mut self,
        mut client:HTTPClient,
    ){
        let pool_reference=self.pool.clone();
        let thread_id=self.counter;

        let thread_name=format!("C-{}",thread_id);
        let client_thread=Builder::new()
                .name(thread_name)
                .stack_size(self.thread_stack_memory)
                .spawn(move||{
                    log!("Got client");
                    match client.handle(){
                        Ok(())=>{
                            let _=client.flush();
                        }
                        Err(e)=>log!("Finished with an error: {:?}",e)
                    }

                    pool_reference.lock().unwrap().remove(&thread_id);
                })
                .unwrap();

        self.pool.lock().unwrap().insert(self.counter,client_thread);
        self.counter+=1;
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if let Some(name)=std::thread::current().name(){
            print!("{} ",name);
            println!($($arg)*);
        }
    };
}