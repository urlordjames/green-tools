#[tokio::main]
async fn main() {
    let directory = green_lib::Directory::from_url("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").await.unwrap();
    directory.upgrade_game_folder(&std::path::PathBuf::from("test")).await;
}