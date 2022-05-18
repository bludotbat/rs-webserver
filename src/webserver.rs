use std::{net::TcpStream, io::Write, fs, error::Error, collections::HashMap};
use crate::HTTPRequest;
use mlua::prelude::*;

fn hash_map_to_lua_table<'a>(vm: &'a Lua, data: &'a HashMap<String, String>) -> Result<LuaTable<'a>, Box<dyn Error>> {
    let table = vm.create_table()?;

    for (key, value) in data {
        table.set(key.to_string(), value.to_string())?;
    }

    return Ok(table);
}

pub(crate) fn handle_http_request(request: &mut HTTPRequest, mut socket : TcpStream) -> Result<(), Box<dyn Error>> {
    if request.endpoint.starts_with("/static/") {
        let data = fs::read(&format!("static/{}", request.endpoint.replace("/static/", "")));

        let mut result_str = String::new();
        match data {
            Ok(file) => {
                result_str += "HTTP/1.1 200 OK\nServer: RS-Webserver\nAccept-Ranges: bytes\n";
                result_str += &format!("Content-Length: {}\n", file.len()).to_string();
                result_str += "Content-Type: bytes\n\n";
                socket.write(result_str.as_bytes())?;
                socket.write(&file)?;
                socket.flush()?;
            },
            Err(_) => {
                let data = "Failed to find static resource";
                result_str += "HTTP/1.1 200 OK\nServer: RS-Webserver\nAccept-Ranges: bytes\n";
                result_str += &format!("Content-Length: {}\n", data.len()).to_string();
                result_str += "Content-Type: bytes\n\n";
                result_str += &data;
                socket.write(result_str.as_bytes())?;
                socket.flush()?;
            },
        };

        return Ok(());
    }

    let vm = Lua::new();
    let request_info = vm.create_table()?;
    request_info.set("method", request.method.to_string())?;
    request_info.set("endpoint", request.endpoint.to_string())?;

    if !request.headers.is_empty() {
        request_info.set("headers", hash_map_to_lua_table(&vm, &request.headers)?)?;
    }
    if !request.request.is_empty() {
        request_info.set("prams", hash_map_to_lua_table(&vm, &request.request)?)?;
    }
    if !request.cookies.is_empty() {
        request_info.set("cookies", hash_map_to_lua_table(&vm, &request.cookies)?)?;
    }
    if !request.body_prams.is_empty() {
        request_info.set("body", hash_map_to_lua_table(&vm, &request.body_prams)?)?;
    }

    vm.globals().set("request", request_info)?;
    vm.globals().set("result", vm.create_table()?)?;

    let contents = fs::read_to_string("server.lua").expect("Failed to read file");
    vm.load(&contents).exec()?;

    let result_table: LuaTable = vm.globals().get("result")?;
    let status: u32 = result_table.get("status")?;
    let data: String = result_table.get("data")?;

    let mut result_str = String::new();
    result_str += &format!("HTTP/1.1 {} {}\nServer: RS-Webserver\nAccept-Ranges: bytes\n", status, if status == 200 {"OK"} else {"ERROR"}).to_string();

    if result_table.contains_key("headers")? {
        let headers: LuaTable = result_table.get("headers")?;
        for data in headers.pairs::<String, String>() {
            if data.is_err() {continue;}
            let pair = data.unwrap();
            result_str += &format!("{}: {}\n", pair.0, pair.1);
        }
    }

    if result_table.contains_key("cookies")? {
        let cookie_table: LuaTable = result_table.get("cookies")?;
        for data in cookie_table.pairs::<String, String>() {
            if data.is_err() {continue;}
            let pair = data.unwrap();
            result_str += &format!("Set-Cookie: {}={}\n", pair.0, pair.1);
        }
    }

    result_str += &format!("Content-Length: {}\n", data.len()).to_string();
    result_str += "Content-Type: bytes\n\n";
    result_str += &data;
    socket.write(result_str.as_bytes())?;
    socket.flush()?;
    Ok(())
}