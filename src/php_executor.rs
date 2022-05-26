use std::{
    collections::HashMap,
    process::Command,
};

use crate::{
    PHP_PATH,

    HTTPMethod
};

pub fn execute(file_path:&str,method:HTTPMethod,args:&HashMap<String,String>)->Vec<u8>{
    let mut command=Command::new(unsafe{&PHP_PATH});
    command.args(["-c","./php_config/php.ini"]);
    command.arg(file_path);

    match method{
        HTTPMethod::Get=>{command.arg("GET");}
        HTTPMethod::Post=>{command.arg("POST");}
    }

    for (key,value) in args{
        command.arg(format!("{}={} ",key,value));
    }

    let output=command.output().unwrap();

    if !output.stderr.is_empty(){
        unsafe{println!("{}",String::from_utf8_unchecked(output.stderr))}
    }

    output.stdout
}