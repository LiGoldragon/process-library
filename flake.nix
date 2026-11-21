{
  description = "The process library, and the Tally Nexus that shows it.";

  inputs.base.url = "github:LiGoldragon/build-base";

  outputs = { self, base }: base.lib.workspaceOutputs { cargoNix = ./Cargo.nix; };
}
