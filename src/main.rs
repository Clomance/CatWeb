#![allow(non_snake_case,non_upper_case_globals,dead_code,invalid_value)]
mod http_client;
use http_client::{
    HTTPClient,
    HTTPMethod
};

mod dynamic_threading;
use dynamic_threading::DynamicThreading;

mod php_executor;

#[cfg(target_os="linux")]
use signal_hook::{
    iterator::Signals,
    consts::signal::SIGTERM
};

#[cfg(target_os="linux")]
use std::{
    net::{
        TcpStream
    }
};

use std::{
    io::Read,
    net::{
        Ipv4Addr,
        IpAddr,
        SocketAddr,
        TcpListener,
    },
    time::Duration,
    path::Path,
};

// Время ожидания получения/отправки данных
const default_client_rw_timeout:Duration=Duration::from_micros(2500);

const DEFAULT_SOURCE_DIRECTORY:&'static str=".";
// Папка с файлами веб приложения
pub static mut SOURCE_DIRECTORY:String=String::new();

const DEFAULT_PHP_PATH:&'static str="php";
pub static mut PHP_PATH:String=String::new();

pub static mut running:bool=true;

fn main(){
    println!("Starting...");

    let mut args=std::env::args();

    let _path=args.next().unwrap();

    let mut address=Ipv4Addr::new(127,0,0,1);
    let mut port=8080u16;
    if let Some(args_address_port)=args.next(){
        if let Some((args_address,args_port))=args_address_port.split_once(":"){
            let address_parts:Vec<u8>=args_address.split(".")
                    .map(|part|part.trim().parse().unwrap()).collect();

            if address_parts.len()==4{
                address=Ipv4Addr::new(
                    address_parts[0],
                    address_parts[1],
                    address_parts[2],
                    address_parts[3]
                );

                port=args_port.trim().parse().unwrap();
            }
        }
    }

    println!("Address IPv4 - {}:{}",address,port);

    unsafe{
        SOURCE_DIRECTORY.push_str(DEFAULT_SOURCE_DIRECTORY);
        if let Some(source)=args.next(){
            let path=Path::new(&source);
            if path.is_dir(){
                SOURCE_DIRECTORY=source;
            }
        }
        else if let Ok(source)=std::env::var("SOURCE_DIRECTORY"){
            let path=Path::new(&source);
            if path.is_dir(){
                SOURCE_DIRECTORY=source;
            }
        }
        std::env::set_var("SOURCE_DIRECTORY",&SOURCE_DIRECTORY);

        println!("Source - {}",SOURCE_DIRECTORY);
    }

    unsafe{
        PHP_PATH.push_str(DEFAULT_PHP_PATH);
        if let Some(php)=args.next(){
            let path=Path::new(&php);
            if path.is_file(){
                PHP_PATH=php;
            }
        }
        else if let Ok(php)=std::env::var("PHP_PATH"){
            let path=Path::new(&php);
            if path.is_file(){
                PHP_PATH=php;
            }
        }

        println!("PHP - {}",PHP_PATH);
    }

    let mut thread_limit:usize=128;
    if let Ok(limit)=std::env::var("THREAD_LIMIT"){
        thread_limit=limit.parse().unwrap();
    }

    let mut thread_stack_memory:usize=2048;
    if let Ok(memory)=std::env::var("THREAD_STACK_MEMORY"){
        thread_stack_memory=memory.parse().unwrap();
    }

    let mut thread_pool=DynamicThreading::new(thread_limit,thread_stack_memory);

    let ip=IpAddr::V4(address);
    let address=SocketAddr::new(ip,port);

    let server_socket=TcpListener::bind(address).unwrap();

    let mut buffer=[0u8;4096];

    #[cfg(target_os="linux")]{
        let mut signals=Signals::new(&[SIGTERM]).unwrap();
        std::thread::spawn(move||{
            for _ in &mut signals{
                println!("SIGTERM received");
                unsafe{running=false}
                TcpStream::connect(address).unwrap();
            }
        });
    }

    println!("Set up");

    println!("Listenining...");
    while unsafe{running}{
        match server_socket.accept(){
            Ok((mut client_socket,_address))=>{
                println!("Got connection");
                // Установка времени ожидания для отправки и принятия данных от клиента
                if client_socket.set_read_timeout(Some(default_client_rw_timeout)).is_ok() &&
                        client_socket.set_write_timeout(Some(default_client_rw_timeout)).is_ok()
                {
                    if let Ok(bytes)=client_socket.read(&mut buffer){
                        if let Ok(client)=HTTPClient::new(client_socket,&buffer[0..bytes]){
                            thread_pool.handle_client(client);
                        }
                    }
                }

                println!("Handled");
            },
            Err(_)=>{

            }
        }
    }

    println!("Shutdown");
}
