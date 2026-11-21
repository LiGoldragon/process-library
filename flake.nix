{
  description = "The process library, and the Tally Nexus that shows it.";

  inputs.base.url = "github:LiGoldragon/build-base";
  inputs.ethos-zero.url = "github:LiGoldragon/ethos-zero";

  outputs =
    {
      self,
      base,
      ethos-zero,
    }:
    base.lib.workspaceOutputs {
      cargoNix = ./Cargo.nix;
      extraChecks =
        {
          pkgs,
          toolchain,
          workspace,
          ...
        }:
        let
          inherit (pkgs) lib;
          tally = workspace.workspaceMembers."tally-process".build;
          process = lib.findFirst (
            crate: crate.crateName == "process-library"
          ) (throw "Tally must link the process library") tally.dependencies;
          dependencyPaths = lib.concatMapStringsSep " " (
            crate: "-L dependency=${lib.getLib crate}/lib"
          ) tally.completeDeps;
        in
        {
          tally-generated =
            pkgs.runCommand "tally-generated"
              {
                compiler = ethos-zero.packages.${pkgs.stdenv.hostPlatform.system}.ethos-zero;
                roots = ./ethos;
                committed = ./tally-process/src/generated;
              }
              ''
                rootsDatom=""
                for root in "$roots"/*.ethos; do rootsDatom="$rootsDatom «$root»"; done
                reply=$("$compiler/bin/ethos-zero" "Generate.{ [ $rootsDatom ] «$TMPDIR/generated» }")
                case "$reply" in
                  Generated.*) echo "$reply" ;;
                  *) echo "$reply" >&2; exit 1 ;;
                esac
                diff -ru "$committed" "$TMPDIR/generated"
                touch "$out"
              '';
          gift-permissions =
            pkgs.runCommand "tally-gift-permissions"
              {
                nativeBuildInputs = [
                  toolchain
                  pkgs.jq
                ];
                fixture = ./tests/compile-fail/giving.rs;
              }
              ''
                tally=(${lib.getLib tally}/lib/libtally_process-*.rlib)
                process=(${lib.getLib process}/lib/libprocess_library-*.rlib)
                compile() {
                  rustc --edition=2024 --crate-type=lib --emit=metadata \
                    ${dependencyPaths} --extern "tally_process=''${tally[0]}" \
                    --extern "process=''${process[0]}" "$@" "$fixture"
                }
                compile
                echo "AddendProcess can give Add"
                if compile --cfg wrong_process --error-format=json 2> rejection.json; then
                  echo "SumProcess unexpectedly compiled with the same Add gift" >&2
                  exit 1
                fi
                jq -es '
                  [ .[] | select(.level == "error" and .code != null) ] |
                  length == 1 and
                    .[0].code.code == "E0277" and
                    (.[0].message | contains("Add: Givable<SumProcess>"))
                ' rejection.json
                jq -r 'select(.code.code == "E0277") | .message' rejection.json
                mkdir -p "$out"
                cp rejection.json "$out/"
              '';
        };
    };
}
