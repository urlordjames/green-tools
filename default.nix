let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.1.2" = "sha256-XaGbHSFMZI7WOSgycjNHFHrbWBkFkRXJjO5aRSNCPhE=";
		};
	};
}
