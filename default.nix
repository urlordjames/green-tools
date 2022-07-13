let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.1.1" = "sha256-pW/QNttrE+FspK49Rr+6skiN9LH6nZQoaZpG1/dNMKc=";
		};
	};
}
