let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.0";

	src = ./.;

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.2.0" = "sha256-nOypBVP/ggCqVcn9qi4f36CmCPl3cyRG/o9xXckgvEc=";
		};
	};
}
