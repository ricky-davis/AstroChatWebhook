
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::State;
use rocket::config::{Config, Environment, LoggingLevel};

use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use serde_json::json;
use rand::seq::SliceRandom;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;



#[get("/?<evt>&<msg>&<name>")]
fn webhook(app_data: State<AppData>, evt: String, mut msg: String, name: String) {
    //println!("{} -- {} -- {}", evt, name, msg);
    let mut whname = name.clone();
    let mut console_buffer="".to_string();

    let dwhd = match evt.as_str(){
        "join" => 'j',
        "leave" => 'l',
        "chat" => 'c',
        "cmd" => 'd',
        _ => 'z'
    };

    match dwhd {
        'j' =>{
            whname = app_data.my_name.to_string();
            msg = format!("{} {}", &name, "has joined the server.");
            console_buffer = format!("{} has joined the server!", &name);
        },
        'l' => {
            whname = app_data.my_name.to_string();
            msg = format!("{} {}", &name, "has left the server.");
            console_buffer = format!("{} has left the server!", &name);
        },
        'c' => console_buffer = format!("{} said: {}", &whname, &msg),
        'd' => console_buffer = format!("{} did command: {}", &whname, &msg),
        _ => ()

    }

    let mut lock = app_data.avatar_dict.lock().expect("lock shared data");
    if !lock.contains_key(&whname){
        let mut rng: Pcg64 = Seeder::from(&whname).make_rng();
        let sample: Vec<_> = app_data.avatar_themes
        .choose_multiple(&mut rng, 1)
        .collect();
        let name_theme = sample.first().unwrap().to_string();
        let avatar_url = format!("https://www.tinygraphs.com/squares/{}?theme={}&numcolors=4&size=220&fmt=png", &whname, &name_theme);
        lock.insert(whname.clone(), avatar_url.clone());
    }

    let new_avt = lock.get(&whname).unwrap().to_string();
    let message = format!("{} {}",app_data.emoji_map.get(&dwhd).unwrap(),msg);

    
    let request_obj = json!({
            "content": message,
            "username": whname,
            "avatar_url": new_avt,
            "allowed_mentions": {
                "parse": []
            }
        });

    let client = reqwest::blocking::Client::new();
    let _res = client.post(app_data.discord_url.as_str())
        .json(&request_obj)
        .send().unwrap();
    
    println!("{}", console_buffer);
    //println!("{:?}",res);
}


struct AppData {
    my_name: String,
    discord_url: String,
    avatar_themes: Vec<String>,
    avatar_dict: Mutex<HashMap<String, String>>,
    emoji_map: HashMap<char, String>
}

impl AppData {
    pub fn new(name: String, discord_url: String, avatar_themes:Vec<String>, avatar_dict: Mutex<HashMap<String, String>>, emoji_map: HashMap<char, String>) -> AppData{
        AppData {
            my_name: name,
            discord_url: discord_url,
            avatar_themes: avatar_themes,
            avatar_dict: avatar_dict,
            emoji_map: emoji_map
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    let my_name = &args[1];
    let my_url = &args[2];
    let my_port = args[3].parse::<u16>().unwrap();
    let discord_url = &args[4];


    let mut tem = HashMap::new();
    tem.insert('j', ":wave:");           // Join
    tem.insert('l', ":x:");              // Leave
    tem.insert('s', ":floppy_disk:");    // Save
    tem.insert('b', ":recycle:");        // Backup
    tem.insert('c', ":speech_balloon:"); // Chat
    tem.insert('d', ":bangbang:");       // CMD
    let emoji_map = tem.into_iter().map( |(key, value)| (key, value.to_string()) ).collect(); // convert from str to String


    let tat = vec![
        "frogideas",
        "sugarsweets",
        "heatwave",
        "daisygarden",
        "seascape",
        "summerwarmth",
        "bythepool",
        "duskfalling",
        "berrypie"
        ];
    let avatar_themes: Vec<String> = tat.into_iter().map(|s| s.to_string()).collect(); // convert from str to String



    let app_data = AppData::new(my_name.clone(),discord_url.clone(), avatar_themes, Mutex::new(HashMap::new()), emoji_map);


    let main_mount = format!("/{}", my_url);

    let config = Config::build(Environment::Staging)
        .port(my_port)
        .log_level(LoggingLevel::Off)
        .finalize()
        .unwrap();

    println!("Running the AstroChatWebhook forwarder..");
    println!("Waiting for input at http://localhost:{}/{}", my_port, my_url);
    
    rocket::custom(config).manage(app_data).mount(&main_mount, routes![webhook]).launch();
}