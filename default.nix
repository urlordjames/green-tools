let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.2.0" = "sha256-0jpM1U3iBXPUWh7jULvwpCf0N/UC2yXYTsZyhqPBaXQ=";
		};
	};
}
