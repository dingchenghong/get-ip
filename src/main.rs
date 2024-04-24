use std::{path::Path, io::{self, Write, Read}, fs::File, process::Command};
use chrono::Local;
use path_absolutize::*;
use hyper::{Client, body::HttpBody};
use regex::Regex;
use tokio::io::{stdout, AsyncWriteExt as _};

// 给出一个文件，获取它的所在目录的绝对目录
// 如果目录不存在，则创建一下
// Result返回一个元组
// 元组的第一个元素是文件所在目录
// 元组的第二个元素是文件的绝对路径
// 元组的第三个元素是指第一次创建
fn get_file_path(file: &str) -> Result<(String, String, bool), io::Error> {
    let mut path = "".to_string();
    // 判断文件是否有/目录符号
    let position = match file.find("/") {
        Some(x) => x,
        None => 1000000,
    };
    // 没有/目录符号，即当前目录
    if position == 1000000 {
        path = format!("{}{}", "./", file).to_string();
    } else {
        path = file.to_string();
    }

    // 判断是否是当前目录或上级目录
    if file.starts_with(".") || file.starts_with("..") {
        let new_path = Path::new(file);
        path = new_path.absolutize().unwrap().to_str().unwrap().to_string();
    }

    // 判断文件是不是以~/开头，即用户家目录
    if file.starts_with("~/") {
        let home_path = dirs::home_dir();
        let new_path = file.replace("~", home_path.unwrap().to_str().unwrap());
        path = new_path;
    }
    // 从output_file中截取出真正的目录，然后创建它
    let position = path[..].rfind("/");
    let p = match position {
        Some(x) => x,
        None => 0,
    };
    let file_path = &path[0..p + 1];
    let file_name = &path[p + 1..path.len()];
    let p = Path::new(file_path);
    let mut flag = false;
    if !p.exists() {
        std::fs::create_dir_all(file_path)?;
        flag = true;
    }
    let real_path = format!("{}{}", file_path, file_name);
    Ok((file_path.to_string(),real_path, flag))
}

// 执行命令
fn exe(cmd: &str) {
    println!("cmd is: {}", cmd);
    let output = Command::new("sh").arg("-c").arg(cmd.to_string()).output().expect("sh exec error");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    // let s = String::from_utf8_lossy(&output.stdout);
    let s = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };
    println!("{}", s)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let uri = "http://txt.go.sohu.com/ip/soip".parse()?;
    let mut resp = client.get(uri).await?;
    println!("响应状态码:{:?}", resp.status());
    // And now...
    while let Some(chunk) = resp.body_mut().data().await {
        // stdout().write_all(&chunk?).await?;
        let b = &chunk?;
        let body = String::from_utf8_lossy(&b);
        println!("响应内容是:{}", body);
        // 用正则取出ip
        let reg = Regex::new(r"\d+.\d+.\d+.\d+")?;
        let mut ip = String::new();
        for cap in reg.captures_iter(&body) {
            ip = cap[0].to_string();
        }
        if ! ip.is_empty() {
            println!("ip:{}", ip);
            let fmt = "%Y-%m-%d %H:%M:%S";
            let now = Local::now().format(fmt);
            let cmd = format!(r#"echo "hello world,wq vb ip address is:{}" | heirloom-mailx -s "hello ding --- {}" 2857000511@qq.com"#, ip, now);
            // 读取ip记录文件
            let real_path = get_file_path("~/office-ip/ip-file").unwrap();
            // 是不是第一次,第一次的话立即发送通知
            let first_time = real_path.2;
            let f = real_path.clone();
            if first_time {
                println!("first time");
                exe(&cmd);
                let mut file = File::create(real_path.1).expect("创建文件失败...");
                file.write_all(ip.as_bytes()).expect("数据写入文件失败...");
            } else {
                // 读取文件内容
                let mut file = std::fs::File::open(f.1).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                print!("ip文件里的内容是:{}", content);
                if ip != content {
                    println!("ip变更了,发送通知");
                    exe(&cmd);
                    let mut file = File::create(real_path.1).expect("创建文件失败...");
                    file.write_all(ip.as_bytes()).expect("数据写入文件失败...");
                }
            }
            
        } else {
            println!("获取ip失败...");
        }
    }
    Ok(())
}
