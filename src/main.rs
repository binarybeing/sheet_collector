use actix_cors::Cors;
use actix_files as afs;
use actix_web::{get,post,web,App,HttpResponse,HttpServer,Responder,HttpRequest};
use std::fs;
use std::net::UdpSocket;
use serde_json::{Result, Value};
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use wd_log;
use std::collections::HashMap;
use sled;
use serde::{Deserialize, Serialize};
mod table_page;


#[post("/submit")]
async fn submit(db :web::Data<sled::Db>, req_body:String) -> impl Responder{
    println!("get input = {}",req_body);
    wd_log::log_debug_ln!("submit = {}",req_body);
    let res = do_submit(db, req_body.as_str());
    match res  {
        Ok(msg) => HttpResponse::Ok().body("提交成功"),
        _=> HttpResponse::Ok().body("提交失败")
    }
    
}

fn do_submit(db :web::Data<sled::Db>, json_string :&str) -> Result<String> {
    println!("get input 2= {}",json_string);
    let v: Value = serde_json::from_str(json_string)?;
    //println!("get input 3 = {:?}",v);
    let obj = v.as_object();
    
    let obj = obj.unwrap();
    //println!("json obj={:?}",obj);
    let task_name = obj.get("task_name").unwrap();
    
    

    let task_name = task_name.as_str().unwrap();
    let db = db.open_tree(task_name).unwrap();
    let mut to_order = Vec::new();
    for a in obj {
        let key = a.0;
        let index = json_string.find(key).unwrap();
        to_order.push((index,key));
    }
    to_order.sort_by(|a, b| a.0.cmp(&b.0));
    let mut order_config = Vec::new();
    for tup in to_order{
        order_config.push(tup.1);
    }
    match db.get("table_config").unwrap(){
        Some(config) =>{
            let config = match std::str::from_utf8(config.as_ref()) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let mut config_map: HashMap<String,String> = match serde_json::from_str(config) {
                Ok(v)=> v,
                _=> panic!("trans json error")
            };
            config_map.insert(String::from("order_config"), serde_json::to_string(&order_config).unwrap());
            let config = serde_json::to_string(&config_map).unwrap();
            db.insert("table_config".as_bytes(),sled::IVec::from(config.as_bytes()));
        },
        None =>{
            let mut config_map = HashMap::new();
            config_map.insert("order_config", serde_json::to_string(&order_config).unwrap());
            let config = serde_json::to_string(&config_map).unwrap();
            db.insert("table_config".as_bytes(),sled::IVec::from(config.as_bytes()));
        }
    }
    
    //println!("insert task_name ={}",task_name);
    let res = db.update_and_fetch(task_name.as_bytes(),|value_opt| {
        if let Some(existing) = value_opt {
            let s = match std::str::from_utf8(existing) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            //println!("database value={}",s);
            let new_vec: Vec<Value> = match serde_json::from_str(s) {
                Ok(v)=> v,
                _=> panic!("trans json error")
            };
            let mut new_vec = Vec::from(new_vec.clone());
            new_vec.push(v.clone());
            //println!("json obj new_vec={:?}",new_vec);
            let s = serde_json::to_string(&new_vec);
            let s: String = s.unwrap();
            Some(sled::IVec::from(s.as_bytes()))
        }else {
            let new_vec:&mut Vec<Value> = &mut Vec::new();
            new_vec.push(v.clone());
            let s = serde_json::to_string(&new_vec);
            let s: String = s.unwrap();
            Some(sled::IVec::from(s.as_bytes()))
            
        }
    });

    return Ok(String::from("good"));
}

#[get("/show/{task_name}/{title_name}")]
async fn show(req: HttpRequest) -> impl Responder {
    let mut task_name = String::from("../");
    let task_name_param = req.match_info().get("task_name").unwrap();
    task_name.push_str(task_name_param);
    task_name.push_str("/form_content.html");
    println!("show taske_name={}",task_name);
    let content = fs::read_to_string(task_name).expect("Something went wrong reading html file");
    let mut res = String::from(HTML_HEAD);
    res.push_str(FORM_HEAD);
    let title_name = req.match_info().get("title_name").unwrap();
    res.push_str(format!("<h1 class='col-sm-12' style='text-align:center'>{}</h1>",title_name).as_str()); 
    res.push_str(format!("<input type='hidden' name='task_name' value='{}'>",task_name_param).as_str()); 
    res.push_str(content.as_str());
    res.push_str(FORM_TAIL);
    res.push_str(SCRIPT);
    res.push_str(HTML_TAIL);
    HttpResponse::Ok().body(res)
}

#[derive(Serialize, Deserialize)]
struct TableObj {
    name: String,
    rows: Value,
    config:Value
}

#[get("/show_excel/{task_name}/{title_name}")]
async fn show_excel(db:web::Data<sled::Db>, req: HttpRequest) -> impl Responder {

    if is_not_local_host(&req) {
        return HttpResponse::Ok().body("no permission");
    }
    let task_name_param = req.match_info().get("task_name").unwrap();
    println!("show excel task name={}",task_name_param);
    let db = db.open_tree(task_name_param).unwrap();
    //println!("vv={:?}", json_obj);
    // let json = match serde_json::from_str(a){
    //     Ok(obj)=> obj,
    //     _=> panic!("")
    // };
    //let a = match String::from(std::str::from_utf8(db.get(task_name_param.as_bytes()).unwrap())) {
    let a = match db.get(task_name_param.as_bytes()).unwrap(){
        Some(v) => {
            //println!("v={:?}",v);
            String::from(std::str::from_utf8(v.as_ref()).unwrap())
        },
        _=> String::from("[]")
    };
    let json = match serde_json::from_str(a.as_str()){
        Ok(obj)=> obj,
        _=> panic!("")
    };

    let a = match db.get("table_config").unwrap(){
        Some(v) => {
            String::from(std::str::from_utf8(v.as_ref()).unwrap())
        },
        _=> String::from("[]")
    };
    let config_json = match serde_json::from_str(a.as_str()){
        Ok(obj)=> obj,
        _=> panic!("")
    };
    let res = TableObj{
        name:String::from(task_name_param),
        rows:json,
        config:config_json
    };
    HttpResponse::Ok().json(res)
}

fn is_not_local_host(req: &HttpRequest)-> bool{
    let addr = req.peer_addr().unwrap();
    let ip = addr.ip().to_string();
    if ip != get_host().unwrap() {
        true
    }else{
        false
    }
}
#[get("/jump/{task_name}/{task_title}")]
async fn jump(req :HttpRequest) -> impl Responder {
    let task_name = req.match_info().get("task_name").unwrap();
    let task_title = req.match_info().get("task_title").unwrap();
    let mut content = String::from(HTML_HEAD);
    content.push_str(table_page::content::REFRESH_BUTTON);
    content.push_str(format!("<input type='hidden' id='the_task_name' value='{}'><input type='hidden' id='the_task_title' value='{}'>" ,task_name,task_title).as_str());
    content.push_str(table_page::content::TABLE);
    content.push_str(HTML_TAIL);
    HttpResponse::Ok().body(content)
}

#[get("/")]
async fn indexx(req :HttpRequest) -> impl Responder {
    let file = File::open("../collect_task").unwrap();
    let fin = BufReader::new(file);
    let mut content = String::from("");
    let is_not_local = is_not_local_host(&req);
    println!("is not local={}",is_not_local);
    for line in fin.lines() {
        let line = line.unwrap();
        let vec :Vec<&str> = line.split_whitespace().collect();
        let mut sub_content = String::from("<div style='height:100px'>");
        if !is_not_local {
            sub_content.push_str(format!("<span><a href='/jump/{}/{}'>查看</a>&nbsp;&nbsp;&nbsp;&nbsp;</span>",vec[0],vec[1]).as_str());
        }
        sub_content.push_str(format!("<a href='/show/{}/{}' style='font-size:80px'>{}</a>",vec[0],vec[1],vec[1]).as_str());
        sub_content.push_str("</div>");
        content.push_str(sub_content.as_str());
    }
    if content.is_empty() {
        content.push_str("<div>当前无可收集表单</div>");
    }
    let mut res = String::from(HTML_HEAD);
    res.push_str(content.as_str());
    res.push_str(HTML_TAIL);
    HttpResponse::Ok().body(res)
}

#[get("/admin")]
async fn admin(req :HttpRequest) -> impl Responder {
    if is_not_local_host(&req) {
        return HttpResponse::Ok().body("无权限");
    }
    let file = File::open("../collect_task").unwrap();
    let fin = BufReader::new(file);
    let mut content = String::from("");
    for line in fin.lines() {
        let line = line.unwrap();
        let vec :Vec<&str> = line.split_whitespace().collect();
        let sub_content = format!("<div style=\"height:100px\"><a href=\"/show_excel/{}/{}\" style=\"font-size:80px\">{}</a></div>",vec[0],vec[1],vec[1]);
        content.push_str(sub_content.as_str());
    }
    if content.is_empty() {
        content.push_str("<div>当前无可收集表单</div>");
    }
    let mut res = String::from(HTML_HEAD);
    res.push_str(content.as_str());
    res.push_str(HTML_TAIL);
    HttpResponse::Ok().body(res)
}

async fn manual_hello() -> impl Responder{
    HttpResponse::Ok().body("Hey there!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>{
    let mut host = get_host().unwrap();
    host.push_str(":14324");
    let mut origin_host = String::from("http://");
    origin_host.push_str(host.as_str());
    println!("host={}  origin={}",host,origin_host);
    wd_log::output_to_file("../server.log");
    let mut map:HashMap<String, Value> = HashMap::new();
    let mut sled_tree:sled::Db = sled::open("../.database").expect("open database error");
    
    HttpServer::new(move || {
        App::new()
          .wrap(
            Cors::new()
              .allowed_origin("null")
              .allowed_origin(origin_host.as_str())
              .allowed_methods(vec!["GET","POST"])
              .allowed_headers(vec!["Access-Control-Allow-Headers", 
                    "Authorization", "authorization", "X-Requested-With",
                    "Content-Type", "content-type", "Origin", "Client-id",
                    "user-agent", "User-Agent", "Accept", "Referer","referer",
                    "Nonce", "signature", "Timestamp","AppKey","x-super-properties",
                    "X-Super-Properties"])
              .max_age(3600)
              .finish(),
          )
          .data(sled_tree.clone())
          .service(show)
          .service(submit)
          .service(indexx)
          .service(admin)
          .service(jump)
          .service(show_excel)
          .route("/hey", web::get().to(manual_hello))
          .service(afs::Files::new("/resource", ".").show_files_listing())
      })
      .workers(4)
      .bind(host.as_str())?
      .run()
      .await
}
pub fn get_host() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => return Some(addr.ip().to_string()),
        Err(_) => return None,
    };
   }

static HTML_HEAD: &'static str = "<!doctype html><html lang=\"zh-CN\">
  <head>
    <meta charset=\"utf-8\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <title>表单页面</title>
    <link rel=\"stylesheet\" href=\"https://stackpath.bootstrapcdn.com/bootstrap/3.4.1/css/bootstrap.min.css\" integrity=\"sha384-HSMxcRTRxnN+Bdg0JdbxYKrThecOKuH5zCYotlSAcp1+c8xmyTe9GYg1l9a69psu\" crossorigin=\"anonymous\">
    <link rel='stylesheet' href='https://unpkg.com/x-data-spreadsheet@1.1.9/dist/xspreadsheet.css'>
  </head>
  <body>";

static HTML_TAIL: &'static str = "<script src=\"https://cdn.jsdelivr.net/npm/jquery@1.12.4/dist/jquery.min.js\" integrity=\"sha384-nvAa0+6Qg9clwYCGGPpDQLVpLNn0fRaROjHqs13t4Ggj3Ez50XnGQqc/r8MhnRDZ\" crossorigin=\"anonymous\"></script>
<script src=\"https://stackpath.bootstrapcdn.com/bootstrap/3.4.1/js/bootstrap.min.js\" integrity=\"sha384-aJ21OjlMXNL5UyIl/XNwTMqvzeRMZH2w8c5cRVpzpU8Y5bApTppSuUkhZXN0VxHd\" crossorigin=\"anonymous\"></script>
<script type=\"text/javascript\" src=\"https://unpkg.com/x-data-spreadsheet@1.0.20/dist/locale/zh-cn.js\"></script>
<script src=\"https://unpkg.com/x-data-spreadsheet@1.1.9/dist/xspreadsheet.js\"></script>
</body>
</html>";

static FORM_HEAD: &'static str = "<form class=\"form-horizontal\" id=\"info_form\" style=\"margin-top:100px\"><div class=\"form-group\">
<input id='time_millisecond' name='time_millisecond' type='hidden'>";
static FORM_TAIL: &'static str = "<div class=\"col-sm-2\"></div>
<div class=\"col-sm-10\">
  <button type=\"button\" class=\"btn btn-success\" onclick=\"submit_form()\">提交</button>
  <div id=\"myAlert1\" style=\"display:none\" class=\"alert alert-warning\">
      <a href=\"#\" class=\"close\" data-dismiss=\"alert\">&times;</a>
      <strong>警告！</strong>提交失败，请检查输入参数是否正确
  </div>
  <div id=\"myAlert2\" style=\"display:none\" class=\"alert alert-warning\">
      <a href=\"#\" class=\"close\" data-dismiss=\"alert\">&times;</a>
      <strong>警告！</strong>提交失败，请检查网络后重试
  </div>
  <div id=\"myAlert3\" style=\"display:none\" class=\"alert alert-success\">
      <a href=\"#\" class=\"close\" data-dismiss=\"alert\">&times;</a>
      <strong>提交成功！</strong>
      </div>
  </div></div></form>";

static SCRIPT: &'static str = "<script type='text/javascript'>
function submit_form() {
    initSerializeObject();
    $('#time_millisecond').val(new Date().getTime());
    let jsonData = $('#info_form').serializeObject();

    if (check(jsonData)) {
        $.ajax({
            type: 'post',
            url: '/submit',
            contentType: 'application/json; charset=utf-8',
            cache: false,
            withCredentials: true,
            async: false,
            data: JSON.stringify(jsonData),
            success: function(resultData) {
                $('#myAlert3').css('display', 'block');
                setTimeout(\"hidden_obj('#myAlert3')\",2000);
            },
            error: function(result) {
                $('#myAlert2').css('display', 'block');
                setTimeout(\"hidden_obj('#myAlert2')\",2000);
            }
        })
    } else {
        $('#myAlert1').css('display', 'block');
        setTimeout(\"hidden_obj('#myAlert1')\",2000);
    }
}
function hidden_obj(id){
    $(id).css('display', 'none');
}
function initSerializeObject() {
    $.fn.serializeObject = function() {
        var o = {};
        var a = this.serializeArray();
        $.each(a,
        function() {
            if (o[this.name]) {
                if (!o[this.name].push) {
                    o[this.name] = [o[this.name]]
                }
                o[this.name].push(this.value || '')
            } else {
                o[this.name] = this.value || ''
            }
        });
        return o
    }
}
</script>
";




