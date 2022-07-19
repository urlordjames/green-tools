use clap::Parser;
use sha2::{Sha256, Digest};
use green_lib::{Directory, File};

#[derive(Parser, Debug)]
struct Args {
	path: std::path::PathBuf,
	base_url: url::Url
}

fn main() {
	let args = Args::parse();

	let mut directory = Directory {
		name: String::new(),
		files: vec![],
		children: vec![]
	};

	let new_directory = to_directory(&args.path, &mut directory, true, args.base_url);

	let manifest_file = std::fs::File::create("manifest.json").unwrap();
	serde_json::to_writer(&manifest_file, &new_directory).expect("cannot serialize manifest");
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
			true => String::from(""),
			false => path.file_name().unwrap().to_str().expect("invalid directory name").into()
		},
		files,
		children
	}
}