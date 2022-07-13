let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.1.1" = "sha256-TP2GR6FiNzokI0y4vd1Ovn3OEYOZcaAi/yPpe0YesaM=";
		};
	};
}
