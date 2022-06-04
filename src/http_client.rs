use crate::SOURCE_DIRECTORY;

use std::{
    net::{
        TcpStream,
        ToSocketAddrs,
    },
    io::{
        Write,
        Read,
        Error,
        ErrorKind,
    },
    fs::OpenOptions,
    path::{
        Path,
        PathBuf
    },
    collections::HashMap,
    time::Duration,
};

#[derive(Clone,Copy)]
pub enum HTTPMethod{
    Get,
    Post,
}

// pub enum Header{
//     Status(u16),
//     Server(String),
//     AccessControl(AccessControl),
//     ContentType(ContentType),
// }

// pub enum AccessControl{
//     Any,
// }

// pub enum ContentType{
//     PlainText,
//     HTML,
//     CSS,
//     PHP,
//     JS,
//     PNG,
//     JPEG,
//     XML,
//     MP4,
// }

const connection_timeout:Duration=Duration::from_millis(2500);

const redirect_header:&'static str="redirect";
const redirect_over_header:&'static str="xxx-over";

pub struct HTTPClient{
    socket:TcpStream,
    request:String,
    method:HTTPMethod,
    path:String,
    args:HashMap<String,String>,
    headers:HashMap<String,String>
}

impl HTTPClient{
    pub fn new(socket:TcpStream,request:&[u8])->Result<HTTPClient,()>{
        let request=unsafe{std::str::from_utf8_unchecked(request).to_string()};
        let mut request_parts=request.split("\r\n");

        println!("{}",request);

        if let Some(start_line)=request_parts.next(){
            let mut start_line_parts=start_line.split(" ");

            if let Some(method)=start_line_parts.next(){
                let method=match method{
                    "GET"=>HTTPMethod::Get,

                    "POST"=>HTTPMethod::Post,

                    _=>HTTPMethod::Get
                };

                if let Some(request_url)=start_line_parts.next(){
                    let request_url:String=urlencoding::decode(request_url).unwrap().into_owned();

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

                    let mut headers=HashMap::new();
                    for header in request_parts{
                        // Пустая строка указывает на начало тела запроса
                        if header.trim().is_empty(){
                            break
                        }
                        let (name,value)=header.split_once(":").unwrap();
                        headers.insert(name.to_lowercase(),value.to_string());
                    }

                    return Ok(
                        Self{
                            request,
                            socket,
                            method,
                            path,
                            args,
                            headers
                        }
                    )
                }
            }
        }

        Err(())
    }

    pub fn handle(&mut self,thread_id:usize)->std::io::Result<()>{
        let mut buffer=Vec::new();

        if let Some(destination)=self.is_redirect(){
            println!("Thread {} Redirected to {}",thread_id,destination);
            self.redirect(&destination)?
        }
        else{
            let content_type;
            let mut execute=false;

            let path=Path::new(&self.path);

            let mut path_buffer=PathBuf::from(path);

            if path.is_dir(){
                content_type="text/html; charset=utf-8";

                path_buffer.push("index.html");

                println!("Thread {} IndexRequested, Directory {:?}",thread_id,self.path);

                // Если файл `index.html` не существует, то пробуем `index.php`
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
                        "html"=>"text/html; charset=utf-8",
                        "png"=>"image/png",
                        "jpeg"|"jpg"=>"image/jpeg",
                        "xml"|"svg"=>"image/svg+xml",
                        "css"=>"text/css; charset=utf-8",
                        "js"=>"text/javascript; charset=utf-8",
                        "mp4"=>"video/mp4",
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
                self.socket.write(b"HTTP/1.1 404 Not Found\r\nServer: CrocoServer\r\n\r\n")?;
                return Ok(())
            }

            let range=if let Some(range_value)=self.headers.get("range"){
                let mut range_value_parts=range_value.split("=");

                let range=if let Some(_data_type)=range_value_parts.next(){
                    if let Some(range)=range_value_parts.next(){
                        let mut range_parts=range.split("-");

                        if let Some(mut range_start)=range_parts.next(){
                            if let Some(mut range_end)=range_parts.next(){
                                range_start=range_start.trim();
                                range_end=range_end.trim();

                                if range_start.is_empty(){
                                    if range_end.is_empty(){
                                        0..0
                                    }
                                    else{
                                        let from_end:usize=range_end.parse().unwrap();
                                        let start=buffer.len()-from_end;
                                        start..buffer.len()
                                    }
                                }
                                else{
                                    let start:usize=range_start.parse().unwrap();
                                    if range_end.is_empty(){
                                        start..buffer.len()
                                    }
                                    else{
                                        let start:usize=range_start.parse().unwrap();
                                        let end:usize=range_end.parse().unwrap();
                                        start..end
                                    }
                                }
                            }
                            else{
                                0..0
                            }
                        }
                        else{
                            0..0
                        }
                    }
                    else{
                        0..0
                    }
                }
                else{
                    0..0
                };

                self.socket.write(b"HTTP/1.1 206 Partial Content\r\nServer: CrocoServer\r\n")?;

                let range_header=format!("Content-Range: bytes {}-{}/{}\r\n",range.start,range.end-1,buffer.len());
                self.socket.write(range_header.as_bytes())?;

                range
            }
            else{
                self.socket.write(b"HTTP/1.1 200 OK\r\nServer: CrocoServer\r\n")?;

                0..buffer.len()
            };

            let content_type_header=format!("Content-Type: {}\r\n",content_type);

            self.socket.write(b"Access-Control-Allow-Origin: *\r\n")?;
            self.socket.write(content_type_header.as_bytes())?;
            self.socket.write(b"Cache-Control: public\r\n")?;

            self.socket.write(format!("Content-Length: {}\r\n\r\n",buffer.len()).as_bytes())?;
            self.socket.write(&mut buffer[range])?;
        }

        Ok(())
    }

    pub fn is_redirect(&mut self)->Option<String>{
        self.headers.remove(redirect_header)
    }

    pub fn redirect(&mut self,destination:&str)->std::io::Result<()>{
        let mut buffer=Vec::new();

        let mut error=Error::new(ErrorKind::TimedOut,"");

        match ("194.58.117.17",80).to_socket_addrs(){
            Ok(addresses)=>{
                println!("{:?}",addresses);

                for address in (destination,80).to_socket_addrs()?{
                    match TcpStream::connect_timeout(&address,connection_timeout){
                        Ok(mut stream)=>{
                            let redirect_header_start=self.request.find(redirect_header).unwrap();
                            let redirect_header_end=redirect_header_start+redirect_header.len();
                            let redirect_header_range=redirect_header_start..redirect_header_end;
                            self.request.replace_range(redirect_header_range,redirect_over_header);
        
                            stream.write_all(self.request.as_bytes())?;
                            stream.read_to_end(&mut buffer)?;
                            self.socket.write(&buffer)?;
                            return Ok(())
                        }
                        Err(e)=>error=e
                    }
                }

                Err(error)
            }
            Err(e)=>{
                println!("{:?}",e);
                Err(e)
            }
        }
    }

    pub fn is_recursive_redirect(&mut self)->bool{
        if let Some(_)=self.headers.remove(redirect_over_header){
            true
        }
        else{
            false
        }
    }

    #[inline(always)]
    pub fn flush(&mut self)->std::io::Result<()>{
        self.socket.flush()
    }
}