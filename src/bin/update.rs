use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json")]
    url: url::Url,
    path: Option<std::path::PathBuf>
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let directory = green_lib::Directory::from_url(args.url).await.expect("invalid manifest, you fucking dumbass");
    let path = match args.path {
        Some(path) => path,
        None => green_lib::util::minecraft_path()
    };
    directory.upgrade_game_folder(&path).await;
}
