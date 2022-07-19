use clap::Parser;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

#[derive(Parser)]
struct Args {
	#[clap(long, default_value = "https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json")]
	url: url::Url,

	path: Option<std::path::PathBuf>
}

lazy_static::lazy_static! {
	static ref N: AtomicU16 = AtomicU16::new(0);
	static ref TOTAL: AtomicUsize = AtomicUsize::new(0);
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let directory = green_lib::Directory::from_url(args.url).await.expect("invalid manifest");

	let path = match args.path {
		Some(path) => path,
		None => green_lib::util::minecraft_path()
	};

	directory.upgrade_game_folder(&path, |t| {
		TOTAL.store(t, Ordering::SeqCst);
	}, || {
		let n = N.fetch_add(1, Ordering::SeqCst);
		println!("downloaded file {}/{}...", n, TOTAL.load(Ordering::SeqCst));
	}).await;

	println!("finished");
}
