use clap::Parser;

#[derive(Parser)]
struct Args {
	#[clap(long)]
	url: Option<url::Url>,

	path: Option<std::path::PathBuf>
}

const DEFAULT_URL: &str = "https://s3-us-east-2.amazonaws.com/le-mod-bucket/packs.json";

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let directory = match args.url {
		Some(url) => green_lib::Directory::from_url(url).await.expect("invalid manifest"),
		None => green_lib::packs::PacksListManifest::from_url(DEFAULT_URL).await.expect("invalid packs list")
			.get_featured_pack_metadata().expect("can't get featured pack")
			.to_directory().await.expect("invalid featured pack")
	};

	let path = match args.path {
		Some(path) => path,
		None => green_lib::util::minecraft_path()
	};

	let (tx, mut rx) = tokio::sync::mpsc::channel(128);
	let handle = tokio::spawn(async move {
		directory.upgrade_game_folder(&path, Some(tx)).await;
	});

	tokio::spawn(async move {
		let mut counter = None;
		let mut total = None;

		while let Some(msg) = rx.recv().await {
			match msg {
				green_lib::UpgradeStatus::Tick => {
					counter = counter.map(|val| {
						let new_val = val + 1;
						let total = total.expect("total should be set");
						println!("{}/{} ({:.1}%)", new_val, total, (new_val as f32 / total as f32) * 100.0);
						new_val
					});
				},
				green_lib::UpgradeStatus::Length(size) => {
					counter = Some(0);
					total = Some(size);
				}
			}
		}
	});

	handle.await.unwrap();

	println!("finished");
}
