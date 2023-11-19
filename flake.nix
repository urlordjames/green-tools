{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable"; # TODO: change to stable once 23.11 releases
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
			craneLib = crane.lib.${system};
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
