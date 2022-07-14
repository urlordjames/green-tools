let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.1.1" = "sha256-DvS9i4jZq65ADTp/zBKLq6ytJgNC12Bfvm9teLrw0bM=";
		};
	};
}
