{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-24.05";
		flake-utils.url = "github:numtide/flake-utils";

		crane = {
			url = "github:ipetkov/crane";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, flake-utils, crane }:
		flake-utils.lib.eachDefaultSystem (system:
			let pkgs = import nixpkgs {
				inherit system;
			};
			craneLib = (crane.mkLib pkgs);
			green-tools = craneLib.buildPackage {
				src = craneLib.cleanCargoSource (craneLib.path ./.);
			}; in {
				devShell = pkgs.mkShell {
					buildInputs = with pkgs; [
						cargo
						clippy
						cargo-deb
					];
				};

				packages.default = green-tools;
			}
		);
}
