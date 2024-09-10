use crate::main_select::Friend;

pub(crate) fn select(friends: Vec<Friend>) {
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