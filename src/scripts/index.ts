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

program
    .command("generate-resolver-operation")
    .description("generate resolver operation")
    .action(async () => {
        require("./generate_resolver_operation");
    });

program
    .command("import-resolver-operation")
    .description("import resolver operation")
    .action(async () => {
        require("./import_resolver_operation");
    });

program.parse(process.argv);
