use rand::Rng;
use std::collections::HashSet;
use std::io;
use std::str::FromStr;
use steamworks::{Client, ClientManager, Friend, FriendFlags};

fn main() {
    println!("Enter your Steam Web API Key:");

    let stdin = io::stdin();
    let mut steam_key = String::new();
    stdin.read_line(&mut steam_key).unwrap();
    steam_key = steam_key.trim().to_owned();

    assert_eq!(
        32,
        steam_key.len(),
        "Steam Web API Key is the wrong length."
    );

    let (client, _) = Client::init_app(480).expect("Failed to initialize steam.");

    let friends = client.friends();

    let friends_list = friends.get_friends(FriendFlags::IMMEDIATE);
    println!("{:?}", friends_list);

    println!("The following is your friends list:");

    for (i, friend_name) in friends_list.iter().map(|f| f.name()).enumerate() {
        println!("{}: {}", i, friend_name);
    }
    println!("\nEnter the numbers of the friends to include in the search separated by spaces.");
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input = input.trim().to_owned();
    let sel_nums: Vec<u8> = input
        .split(' ')
        .map(|num| u8::from_str(num).expect("Cannot parse one of your answers into a valid number"))
        .collect();

    if *sel_nums.iter().max().unwrap() > (friends_list.len() - 1) as u8 {
        panic!("Invalid input numbers given");
    }

    //println!("{:?}", sel_nums);

    let friends_sel: Vec<&Friend<ClientManager>> = friends_list
        .iter()
        .enumerate()
        .filter_map(|(i, friend)| {
            if sel_nums.contains(&(i as u8)) {
                Some(friend)
            } else {
                None
            }
        })
        .collect();

    if friends_sel.is_empty() {
        panic!("No friends selected");
    }

    println!("Friends selected:");
    for friend in &friends_sel {
        println!("{}", friend.name());
    }
    print!("Pulling games...");

    let you_url = format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true", steam_key, client.user().steam_id().raw());

    let result = json::parse(&reqwest::blocking::get(you_url).unwrap().text().unwrap()).unwrap();

    println!(
        "You have {:?} game(s)",
        result["response"]["game_count"].as_u32().unwrap()
    );

    let mut collective_games: HashSet<i64> = result["response"]["games"]
        .members()
        .map(|game| game["appid"].as_i64().unwrap())
        .collect();

    let mut num_friends_retrieved_from = 0u32;

    for friend in &friends_sel {
        let url = format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true", steam_key, friend.id().raw());
        //println!("{}", url);

        let result = json::parse(&reqwest::blocking::get(url).unwrap().text().unwrap()).unwrap();
        if !result["response"].has_key("game_count") {
            println!("{} has no games that you can see. Skipping", friend.name());
            continue;
        }
        println!(
            "{} has {:?} game(s)",
            friend.name(),
            result["response"]["game_count"].as_u32().unwrap()
        );

        let new_set: HashSet<i64> = result["response"]["games"]
            .members()
            .map(|game| game["appid"].as_i64().unwrap())
            .collect();

        collective_games = collective_games.intersection(&new_set).copied().collect();
        num_friends_retrieved_from += 1;
    }

    if num_friends_retrieved_from == 0 {
        panic!("Couldn't retrieve games from any of the friends specified!");
    }

    println!("Found {} game(s) in common", collective_games.len());

    let mut collective_multiplayer_games: Vec<String> = Vec::new();

    for (i, game_id) in collective_games.iter().enumerate() {
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}",
            game_id
        );
        //println!("url: {}", url);

        println!("Fetching game info ({}/{})", i + 1, collective_games.len());

        let resp = json::parse(&reqwest::blocking::get(url).unwrap().text().unwrap()).unwrap();

        // If this is none, it's probably something like a test version for another game in their library.
        if let Some(name) = resp[game_id.to_string()]["data"]["name"].as_str() {
            let categories = resp[game_id.to_string()]["data"]["categories"]
                .members()
                .map(|cat| cat["id"].as_u32().unwrap())
                .collect::<Vec<u32>>();
            let multiplayer = categories.contains(&1u32)
                || categories.contains(&9u32)
                || categories.contains(&32u32);

            if multiplayer {
                collective_multiplayer_games.push(name.to_owned());
            }
        }
    }

    println!(
        "Found {} multiplayer/coop games in common",
        collective_multiplayer_games.len()
    );

    let rand_pick = rand::thread_rng().gen_range(0..collective_multiplayer_games.len() - 1);

    println!(
        "Your random game is {}!",
        collective_multiplayer_games[rand_pick]
    )
}
