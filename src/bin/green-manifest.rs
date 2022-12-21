use clap::Parser;
use sha2::{Sha256, Digest};
use serde::Deserialize;
use green_lib::{Directory, File};

#[derive(Parser, Debug)]
struct Args {
	#[arg(requires("base_url"))]
	path: Option<std::path::PathBuf>,

	base_url: Option<url::Url>,

	#[arg(long, required_unless_present("path"))]
	extras: Option<std::path::PathBuf>
}

#[derive(Deserialize)]
struct CurseForgeExtra {
	file_id: u32
}

#[derive(Deserialize)]
struct CustomExtra {
	url: String
}

#[derive(Deserialize)]
struct Extras {
	curseforge: Option<std::collections::HashMap<String, CurseForgeExtra>>,
	custom: Option<std::collections::HashMap<String, CustomExtra>>
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	let mut directory = Directory {
		name: String::new(),
		files: vec![],
		children: vec![]
	};

	directory = match &args.path {
		Some(path) => to_directory(path, &mut directory, true, args.base_url.unwrap()),
		None => Directory {
			name: String::new(),
			files: vec![],
			children: vec![]
		}
	};

	if let Some(extras) = args.extras {
		let mods_dir = match directory.children.iter_mut().find(|child| child.name == "mods") {
			Some(child) => child,
			None => {
				let mod_dir = Directory {
					name: String::from("mods"),
					files: vec![],
					children: vec![]
				};

				directory.children.push(mod_dir);
				directory.children.last_mut().unwrap()
			}
		};

		let extras: Extras = toml::from_str(&std::fs::read_to_string(extras).unwrap()).unwrap();

		for (file_name, extra) in extras.curseforge.unwrap_or_default() {
			let url = format!("https://mediafilez.forgecdn.net/files/4226/{}/{}", extra.file_id, file_name);

			mods_dir.files.push(File {
				name: file_name,
				sha: get_sha(&url).await,
				url
			});
		}

		for (file_name, extra) in extras.custom.unwrap_or_default() {
			mods_dir.files.push(File {
				name: file_name,
				sha: get_sha(&extra.url).await,
				url: extra.url
			});
		}
	}

	let manifest_file = std::fs::File::create("manifest.json").unwrap();
	serde_json::to_writer(&manifest_file, &directory).expect("cannot serialize manifest");
}

async fn get_sha(url: &str) -> String {
	let resp = reqwest::get(url).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to download {}", url);
	}

	let contents = resp.bytes().await.unwrap();
	format!("{:x}", Sha256::digest(&contents))
}

fn to_directory(path: &std::path::Path, directory: &mut Directory, top_level: bool, url: url::Url) -> Directory {
	let mut children = vec![];
	let mut files = vec![];

	let read_dir = std::fs::read_dir(path).expect("cannot read path");
	read_dir.filter_map(Result::ok).for_each(|file| {
		let file_type = file.file_type().unwrap();
		let mut file_name = file.file_name().into_string().expect("invalid file name");

		if file_type.is_dir() {
			file_name.push('/');
			let new_url = url.join(&file_name).unwrap();
			children.push(to_directory(&file.path(), directory, false, new_url));
		} else if file_type.is_file() {
			let new_url = url.join(&file_name).unwrap();
			let contents = std::fs::read(file.path()).expect("cannot read file");
			let sha = Sha256::digest(contents);

			files.push(File {
				name: file_name,
				sha: format!("{:x}", sha),
				url: new_url.to_string()
			});
		}
	});

	Directory {
		name: match top_level {
			true => String::new(),
			false => path.file_name().unwrap().to_str().expect("invalid directory name").into()
		},
		files,
		children
	}
}
