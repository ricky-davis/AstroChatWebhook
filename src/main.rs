
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::State;
use rocket::config::{Config, Environment};



#[macro_use] extern crate lazy_static;


use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use serde_json::json;
use rand::seq::SliceRandom;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;




lazy_static! {
    
    static ref AVATAR_THEMES: Vec<&'static str> = vec![
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
    static ref DISCORD_WEBHOOK_AVATAR_DICT: Mutex<HashMap<String, String>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };    
    static ref DISCORD_WEBHOOK_EMOJI: Mutex<HashMap<char, &'static str>> = {
        let mut m = HashMap::new();
        m.insert('j', ":wave:");
        m.insert('l', ":x:");
        m.insert('s', ":floppy_disk:");
        m.insert('b', ":recycle:");
        m.insert('c', ":speech_balloon:");
        m.insert('d', ":bangbang:");
        Mutex::new(m)
    };    
}

#[get("/?<evt>&<msg>&<name>")]
fn webhook(app_data: State<AppData>, evt: String, mut msg: String, name: String) {
    println!("{} -- {} -- {}", evt, name, msg);
    

    let mut map = DISCORD_WEBHOOK_AVATAR_DICT.lock().unwrap();
    let player_name = name.clone();

    let dwhd = match evt.as_str(){
        "join" => 'j',
        "leave" => 'l',
        "chat" => 'c',
        "cmd" => 'd',
        _ => 'z'
    };

    if dwhd == 'j'{
        msg = format!("{} {}", &player_name, "has joined the server.")
    }
    if dwhd == 'l'{
        msg = format!("{} {}", &player_name, "has left the server.")
    }


    let whname;
    if dwhd != 'c' && dwhd != 'd' {
        whname = app_data.my_name.to_string();
    }else{
        whname = player_name.clone();
    }

    let new_avt;
    if !map.contains_key(&whname){
        let mut rng: Pcg64 = Seeder::from(&whname).make_rng();
        let sample: Vec<_> = AVATAR_THEMES
        .choose_multiple(&mut rng, 1)
        .collect();
        let name_theme = sample.first().unwrap();

        let avatar_url = format!("https://www.tinygraphs.com/squares/{}?theme={}&numcolors=4&size=220&fmt=png", &whname, &name_theme);
        map.insert(whname.clone(), avatar_url.clone());
    }

    new_avt = map.get(&whname).unwrap().to_string();
    //println!("{:?}", &new_avt);

    let map2 = DISCORD_WEBHOOK_EMOJI.lock().unwrap();


    let message = format!("{} {}",map2.get(&dwhd).unwrap(),msg);

    
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

    //println!("{:?}",res);
}


struct AppData {
    my_name: String,
    discord_url: String
}

impl AppData {
    pub fn new(name: String, discord_url: String) -> AppData{
        AppData {
            my_name: name,
            discord_url: discord_url
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let my_name = &args[1];
    let my_url = &args[2];
    let my_port = args[3].parse::<u16>().unwrap();
    let discord_url = &args[4];

    let app_data = AppData::new(my_name.clone(),discord_url.clone());

    let main_mount = format!("/{}", my_url);

    let config = Config::build(Environment::Staging)
        .port(my_port)
        .finalize()
        .unwrap();

    rocket::custom(config).manage(app_data).mount(&main_mount, routes![webhook]).launch();
}