use clap::Parser;

#[derive(Parser)]
struct Args {
	#[clap(long, default_value = "https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest2.json")]
	url: url::Url,

	path: Option<std::path::PathBuf>
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let directory = green_lib::Directory::from_url(args.url).await.expect("invalid manifest");

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
