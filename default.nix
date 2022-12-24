let pkgs = import <nixpkgs> {};
in pkgs.rustPlatform.buildRustPackage {
	pname = "green-tools";
	version = "0.1.2";

	src = ./.;

	nativeBuildInputs = with pkgs; [
		pkgconfig
	];

	buildInputs = with pkgs; [
		openssl
	];

	cargoLock = {
		lockFile = ./Cargo.lock;
		outputHashes = {
			"green-lib-0.3.0" = "sha256-K5+GNFAvyuOyGE9Awl1jx+ibMz089VtLLa/pvXbz5AA=";
		};
	};
}
