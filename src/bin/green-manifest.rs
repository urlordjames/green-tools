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
	id1: u32,
	id2: u32
}

#[derive(Deserialize)]
struct CustomExtra {
	url: String
}

#[derive(Deserialize, Debug)]
enum ModrinthVersion {
	#[serde(rename = "version")]
	Version(String),

	#[serde(rename = "version_id")]
	VersionId(String)
}

#[derive(Deserialize, Debug)]
struct ModrinthExtra {
	#[serde(flatten)]
	version: ModrinthVersion,
	deps: Option<std::collections::HashMap<String, ModrinthDep>>
}

#[derive(Deserialize, Debug)]
struct ModrinthDep {
	version_id: String,
	deps: Option<std::collections::HashMap<String, ModrinthDep>>
}

#[derive(Deserialize)]
struct Extras {
	curseforge: Option<std::collections::HashMap<String, CurseForgeExtra>>,
	custom: Option<std::collections::HashMap<String, CustomExtra>>,
	modrinth: Option<std::collections::HashMap<String, ModrinthExtra>>
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
			let url = format!("https://mediafilez.forgecdn.net/files/{}/{}/{}", extra.id1, extra.id2, file_name);

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

		for (mod_id, extra) in extras.modrinth.unwrap_or_default() {
			download_modrinth(&get_modrinth_data(&mod_id, &extra).await, mods_dir, extra.deps.as_ref()).await;
		}
	}

	let manifest_file = std::fs::File::create("manifest.json").unwrap();
	serde_json::to_writer(&manifest_file, &directory).expect("cannot serialize manifest");
}

#[async_recursion::async_recursion]
async fn download_modrinth(version: &serde_json::Value, mods_dir: &mut Directory, deps_lock: Option<&'async_recursion std::collections::HashMap<String, ModrinthDep>>) {
	let jar = &version["files"][0];
	let url = jar["url"].as_str().unwrap();

	let dependencies: &Vec<serde_json::Value> = version["dependencies"].as_array().unwrap();
	for dependency in dependencies.iter() {
		match dependency["version_id"].as_str() {
			Some(dep_version) => download_modrinth(&get_modrinth_version(dep_version).await, mods_dir, None).await,
			None => {
				match deps_lock.unwrap_or(&std::collections::HashMap::new()).get(dependency["project_id"].as_str().unwrap()) {
					Some(dep_extra) => download_modrinth(&get_modrinth_version(&dep_extra.version_id).await, mods_dir, dep_extra.deps.as_ref()).await,
					None => panic!("you are required to specify the version of dependency {:?}", dependency["project_id"])
				};
			}
		};
	}

	mods_dir.files.push(File {
		name: jar["filename"].as_str().unwrap().to_string(),
		sha: get_sha(url).await,
		url: url.to_string()
	});
}

async fn get_modrinth_version(version_id: &str) -> serde_json::Value {
	let resp = reqwest::get(format!("https://api.modrinth.com/v2/version/{}", version_id)).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to fetch modrinth version {:?}", version_id);
	}

	serde_json::from_str(&resp.text().await.unwrap()).unwrap()
}

async fn get_modrinth_data(mod_id: &str, extra: &ModrinthExtra) -> serde_json::Value {
	let resp = reqwest::get(format!("https://api.modrinth.com/v2/project/{}/version", mod_id)).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to fetch modrinth versions for {:?}", extra);
	}

	let versions: serde_json::Value = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
	let versions: &Vec<serde_json::Value> = versions.as_array().unwrap();

	versions.iter().find(|v| match &extra.version {
		ModrinthVersion::Version(version) => v["name"] == *version,
		ModrinthVersion::VersionId(version_id) => v["id"] == *version_id
	}).unwrap().clone()
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
