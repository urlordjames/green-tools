let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.1.1" = "sha256-+MME+bJeGLj0SqyNEE18X337Jzrcwfqg4owmXUa+5vk=";
		};
	};
}
