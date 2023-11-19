use clap::Parser;
use sha2::{Sha256, Digest};
use serde::Deserialize;
use green_lib::{Directory, File};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
enum ModrinthDepVersion {
	#[serde(rename = "version_id")]
	VersionId(String),

	#[serde(rename = "ignore")]
	Ignore(bool)
}

#[derive(Deserialize, Debug)]
struct ModrinthDep {
	#[serde(flatten)]
	version: ModrinthDepVersion,
	deps: Option<std::collections::HashMap<String, ModrinthDep>>
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Extras {
	curseforge: Option<std::collections::HashMap<String, CurseForgeExtra>>,
	custom: Option<std::collections::HashMap<String, CustomExtra>>,
	modrinth: Option<std::collections::HashMap<String, ModrinthExtra>>
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	let mut directory = args.path.as_ref().map(|path| {
		to_directory(path, args.base_url.unwrap())
	}).unwrap_or_default();

	if let Some(extras) = args.extras {
		let mods_dir = match directory.children.get_mut("mods") {
			Some(child) => child,
			None => {
				let mod_dir = Directory::default();
				match directory.children.entry(String::from("mods")) {
					Entry::Vacant(vacant) => vacant.insert(mod_dir),
					_ => unreachable!()
				}
			}
		};

		let extras: Extras = toml::from_str(&std::fs::read_to_string(extras).unwrap()).unwrap();

		for (file_name, extra) in extras.curseforge.unwrap_or_default() {
			let url = format!("https://mediafilez.forgecdn.net/files/{}/{}/{}", extra.id1, extra.id2, file_name);

			mods_dir.files.insert(file_name, File {
				sha: get_sha(&url).await,
				url
			});
		}

		for (file_name, extra) in extras.custom.unwrap_or_default() {
			mods_dir.files.insert(file_name, File {
				sha: get_sha(&extra.url).await,
				url: extra.url
			});
		}

		for (mod_id, extra) in extras.modrinth.unwrap_or_default() {
			download_modrinth(get_modrinth_mod_version(&mod_id, &extra).await, mods_dir, extra.deps.as_ref()).await;
		}
	}

	let manifest_file = std::fs::File::create("manifest2.json").unwrap();
	serde_json::to_writer(&manifest_file, &directory).expect("cannot serialize manifest");
}

#[async_recursion::async_recursion]
async fn download_modrinth(version: ModrinthApiVersion, mods_dir: &mut Directory, deps_lock: Option<&'async_recursion std::collections::HashMap<String, ModrinthDep>>) {
	let jar = version.files.into_iter().find(|f| f.primary).unwrap();

	for dependency in version.dependencies.iter() {
		if dependency.optional() { continue; }
		match &dependency.version_id {
			Some(dep_version) => download_modrinth(get_modrinth_version(dep_version).await, mods_dir, None).await,
			None => {
				match deps_lock.unwrap_or(&std::collections::HashMap::new()).get(&dependency.project_id) {
					Some(dep_extra) => match &dep_extra.version {
						ModrinthDepVersion::VersionId(version_id) => download_modrinth(get_modrinth_version(version_id).await, mods_dir, dep_extra.deps.as_ref()).await,
						ModrinthDepVersion::Ignore(true) => (),
						ModrinthDepVersion::Ignore(false) => panic!("ignore = false has no meaning")
					},
					None => panic!("you are required to specify the version of dependency {:?}", dependency.project_id)
				};
			}
		};
	}

	mods_dir.files.insert(jar.filename, File {
		sha: get_sha(&jar.url).await,
		url: jar.url.to_string()
	});
}

#[derive(Deserialize)]
struct ModrinthApiFile {
	url: String,
	filename: String,
	primary: bool
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ModrinthApiDependencyType {
	Required,
	Optional
}

#[derive(Deserialize)]
struct ModrinthApiDependency {
	version_id: Option<String>,
	project_id: String,
	dependency_type: ModrinthApiDependencyType
}

impl ModrinthApiDependency {
	fn optional(&self) -> bool {
		matches!(self.dependency_type, ModrinthApiDependencyType::Optional)
	}
}

#[derive(Deserialize)]
struct ModrinthApiVersion {
	name: String,
	id: String,
	files: Vec<ModrinthApiFile>,
	dependencies: Vec<ModrinthApiDependency>
}

async fn get_modrinth_version(version_id: &str) -> ModrinthApiVersion {
	let resp = reqwest::get(format!("https://api.modrinth.com/v2/version/{}", version_id)).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to fetch modrinth version {:?}", version_id);
	}

	serde_json::from_str(&resp.text().await.unwrap()).unwrap()
}

async fn get_modrinth_mod_version(mod_id: &str, extra: &ModrinthExtra) -> ModrinthApiVersion {
	let resp = reqwest::get(format!("https://api.modrinth.com/v2/project/{}/version", mod_id)).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to fetch modrinth versions for {:?}", extra);
	}

	let versions: Vec<ModrinthApiVersion> = serde_json::from_str(&resp.text().await.unwrap()).unwrap();

	versions.into_iter().find(|v| match &extra.version {
		ModrinthVersion::Version(version) => &v.name == version,
		ModrinthVersion::VersionId(version_id) => &v.id == version_id
	}).unwrap()
}

async fn get_sha(url: &str) -> String {
	let resp = reqwest::get(url).await.unwrap();
	if !resp.status().is_success() {
		panic!("failed to download {}", url);
	}

	let contents = resp.bytes().await.unwrap();
	format!("{:x}", Sha256::digest(&contents))
}

fn to_directory(path: &std::path::Path, url: url::Url) -> Directory {
	let mut children = HashMap::new();
	let mut files = HashMap::new();

	let read_dir = std::fs::read_dir(path).expect("cannot read path");
	read_dir.filter_map(Result::ok).for_each(|file| {
		let file_type = file.file_type().unwrap();
		let file_name = file.file_name().into_string().expect("invalid file name");

		if file_type.is_dir() {
			let new_url = url.join(&format!("{file_name}/")).unwrap();
			children.insert(file_name, to_directory(&file.path(), new_url));
		} else if file_type.is_file() {
			let new_url = url.join(&file_name).unwrap();
			let contents = std::fs::read(file.path()).expect("cannot read file");
			let sha = Sha256::digest(contents);

			files.insert(file_name, File {
				sha: format!("{:x}", sha),
				url: new_url.to_string()
			});
		}
	});

	Directory {
		files,
		children
	}
}
