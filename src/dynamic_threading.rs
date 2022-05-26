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
    time::{
        Duration,
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

        let client_thread=Builder::new().stack_size(self.thread_stack_memory).spawn(move||{
            client.handle(thread_id).unwrap();
            // Ожидание получения клиентом всех данных
            client.flush().unwrap();

            // Слишком быстрые ответы не воспринимаются Heroku
            std::thread::sleep(Duration::from_millis(2));

            pool_reference.lock().unwrap().remove(&thread_id);
            println!("Thread {} Removed from the thread pool",thread_id);
        }).unwrap();

        self.pool.lock().unwrap().insert(self.counter,client_thread);
        self.counter+=1;
    }
}