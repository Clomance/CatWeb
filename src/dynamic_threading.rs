use super::{
    HTTPClient,
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
                    match client.handle(thread_id){
                        Ok(())=>{
                            let _=client.flush();
                        }
                        Err(e)=>println!("Thread {} Got error: {:?}",thread_id,e)
                    }
                    // Ожидание получения клиентом всех данных

                    pool_reference.lock().unwrap().remove(&thread_id);
                    println!("Thread {} Removed from the thread pool",thread_id);
                })
                .unwrap();

        self.pool.lock().unwrap().insert(self.counter,client_thread);
        self.counter+=1;
    }
}