import { Command, } from "commander";

const program = new Command();

program
    .command("reinstall-all")
    .description("reinstall all canisters")
    .action(async () => {
        require("./reinstall_all");
    });

program
    .command("update-local-config")
    .description("update local config")
    .action(async () => {
        require("./update_local_configs");
    });

program
    .command("load-state")
    .description("load state")
    .action(async () => {
        require("./load_state");
    });

program
    .command("export-state")
    .description("export state")
    .action(async () => {
        require("./export_state");
    });

program.parse(process.argv);