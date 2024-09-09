fn main() {
    cli()
}

use clap::{Arg, Command};
use reqwest;
use serde::Deserialize;

fn cli() {
    let matches = Command::new("chat-cli")
        .version("0.1.0")
        .author("Your Name <your_email@example.com>")
        .about("Handles command line arguments")
        .arg(
            Arg::new("host")
                .long("host")
                .value_parser(clap::value_parser!(String))
                .help("Sets the host address")
                .required(true),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .value_parser(clap::value_parser!(String))
                .help("Sets the user's name")
                .required(true),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .value_parser(clap::value_parser!(String))
                .help("Sets the user's password")
                .required(true),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let name = matches.get_one::<String>("name").unwrap();
    let password = matches.get_one::<String>("password").unwrap();

    let login_url = format!("http://{}:3000/token/login", host);
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "name": name,
            "password": password
        }))
        .send();

    let token = match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Login>() {
                    Ok(Login { msg, data, code: _ }) => {
                        println!("Login successful. Message: {}", msg);
                        println!("Access token: {}", data.access_token);
                        Some(data.access_token)
                    }

                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                        None
                    }
                }
            } else {
                println!("Login failed: HTTP {}", res.status());
                None
            }
        }
        Err(e) => {
            println!("Failed to send login request: {}", e);
            None
        }
    };

    if token.is_none() {
        println!("Login failed. Exiting the program.");
        std::process::exit(1);
    }

    let options = vec!["Friends", "Groups"];
    let selection = dialoguer::Select::new()
        .with_prompt("Please select an option")
        .items(&options)
        .interact()
        .unwrap();

    println!("You selected: {}", options[selection]);
    if options[selection] == "Friends" {
        let client = reqwest::blocking::Client::new();
        let friends_url = format!("http://{}:3000/friend", host);
        let response = client
            .get(&friends_url)
            .header("Authorization", format!("Bearer {}", token.unwrap()))
            .send();

        let friends = match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<Friends>() {
                        Ok(Friends { msg:_, data, code: _ }) => Some(data),
                        Err(e) => {
                            println!("Failed to read response: {}", e);
                            None
                        }
                    }
                } else {
                    println!("Failed to get friends list: HTTP {}", res.status());
                    None
                }
            }
            Err(e) => {
                println!("Failed to send request: {}", e);
                None
            }
        };
        if friends.is_none() {
            println!("Get friends failed. Exiting the program.");
            std::process::exit(1);
        }
        let friends = friends.unwrap();
        let friend_names: Vec<&str> = friends.iter().map(|f| f.name.as_str()).collect();
        let selection = dialoguer::Select::new()
            .with_prompt("Select a friend")
            .items(&friend_names)
            .interact()
            .unwrap();

        let selected_friend = &friends[selection];
        println!(
            "Selected friend - ID: {}, Name: {}",
            selected_friend.id, selected_friend.name
        );
    }
}

// #[derive(Deserialize)]
// enum AppRes {
    #[derive(Deserialize)]
   struct  Login {
        code: i8,
        msg: String,
        data: LoginRes,
    }
    #[derive(Deserialize)]
        struct  Friends {
        code: i8,
        msg: String,
        data: Vec<Friend>,
    }
// }

#[derive(Deserialize)]
struct LoginRes {
    access_token: String,
}

#[derive(Deserialize)]
struct Friend {
    id: i32,
    name: String,
}
