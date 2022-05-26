use crate::SOURCE_DIRECTORY;

use std::{
    net::TcpStream,
    io::{Write,Read},
    fs::OpenOptions,
    path::{
        Path,
        PathBuf
    },
    collections::HashMap,
};

#[derive(Clone,Copy)]
pub enum HTTPMethod{
    Get,
    Post,
}

pub struct HTTPClient{
    pub socket:TcpStream,
    pub method:HTTPMethod,
    pub path:String,
    pub args:HashMap<String,String>,
}

impl HTTPClient{
    pub fn new(socket:TcpStream,headers:&[u8])->Result<HTTPClient,()>{
        let headers=unsafe{std::str::from_utf8_unchecked(headers)};
        let mut split_header=headers.split_whitespace();

        if let Some(method)=split_header.next(){
            let method=match method{
                "GET"=>HTTPMethod::Get,

                "POST"=>HTTPMethod::Post,

                _=>HTTPMethod::Get
            };

            if let Some(request_url)=split_header.next(){
                let mut splited_url=request_url.split("?");

                let mut path=unsafe{SOURCE_DIRECTORY.to_string()};
                path.push_str(splited_url.next().unwrap());

                let mut args=HashMap::new();
                if let Some(url_args)=splited_url.next(){
                    for arg in url_args.split("&"){
                        let mut parts=arg.split("=");
                        let name=parts.next().unwrap();
                        if let Some(value)=parts.next(){
                            args.insert(name.to_string(),value.to_string());
                        }
                    }
                }

                Ok(
                    Self{
                        socket,
                        method,
                        path,
                        args
                    }
                )
            }
            else{
                Err(())
            }
        }
        else{
            Err(())
        }
    }

    pub fn handle(&mut self,thread_id:usize)->std::io::Result<()>{
        let mut buffer=Vec::new();

        let content_type;
        let mut execute=false;

        let path=Path::new(&self.path);

        let mut path_buffer=PathBuf::from(path);

        if path.is_dir(){
            content_type="text/html; charset=utf-8";

            path_buffer.push("index.html");

            println!("Thread {} IndexRequested, Directory {:?}",thread_id,self.path);

            if !path_buffer.exists(){
                path_buffer.pop();
                path_buffer.push("index.php");

                execute=true;
            }
        }
        else{
            println!("Thread {} FileRequested {:?}",thread_id,path);
            let mut file_extension=self.path.rsplit(".");

            content_type=if let Some(extension)=file_extension.next(){
                match extension.to_lowercase().as_str(){
                    "png"=>"image/png",
                    "jpeg"|"jpg"=>"image/jpeg",
                    "xml"|"svg"=>"image/svg+xml",
                    "css"=>"text/css; charset=utf-8",
                    "js"=>"text/javascript; charset=utf-8",
                    "php"=>{
                        execute=true;
                        "text/html; charset=utf-8"
                    },
                    _=>"text/plain; charset=utf-8"
                }
            }
            else{
                "text/plain; charset=utf-8"
            };
        }

        if path_buffer.exists(){
            println!("Thread {} FileExists",thread_id);

            if execute{
                println!("Thread {} FileExecutable",thread_id);

                buffer=crate::php_executor::execute(
                    path_buffer.to_str().unwrap(),
                    self.method,
                    &self.args
                );
            }
            else if let Ok(mut file)=OpenOptions::new().read(true).open(&path_buffer){
                file.read_to_end(&mut buffer)?;
            }
        }   
        else{
            println!("Thread {} FileNotFound",thread_id);
            return self.socket.write_all(b"HTTP/1.1 404 Not Found\r\nServer: CrocoServer\r\n\r\n")
        }

        let content_type_header=format!("Content-Type: {}\r\n",content_type);

        self.socket.write_all(b"HTTP/1.1 200 OK\r\nServer: CrocoServer\r\n")?;
        self.socket.write_all(b"Access-Control-Allow-Origin: *\r\n")?;
        self.socket.write_all(content_type_header.as_bytes())?;
        self.socket.write_all(b"Cache-Control: public\r\n")?;

        self.socket.write_all(format!("Content-Length: {}\r\n\r\n",buffer.len()).as_bytes())?;
        self.socket.write_all(&mut buffer)?;

        Ok(())
    }

    #[inline]
    pub fn flush(&mut self)->std::io::Result<()>{
        self.socket.flush()
    }
}