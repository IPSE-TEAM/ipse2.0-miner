use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};


pub fn build_cli() -> App<'static, 'static> {
    App::new("miner")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("root")
                .short("r")
                .long("root")
                .takes_value(true)
                .default_value(".")
                .help("Directory to use as root of project")
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Path to a config file other than config.toml in the root of project")
        )
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Create a new project")
                .args(&[
                    Arg::with_name("name")
                        .default_value(".")
                        .help("Name of the project. Will create a new directory with that name in the current directory"),
                    Arg::with_name("force")
                        .short("f")
                        .takes_value(false)
                        .help("Force creation of project even if directory is non-empty")
                ]),
            SubCommand::with_name("serve")
                .about("Serve the site. Rebuild and reload on change automatically")
                .args(&[
                    Arg::with_name("address")
                        .short("a")
                        .long("address")
                        .default_value("0.0.0.0")
                        .help("Interface to bind on"),
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .default_value("8888")
                        .help("Which port to use"),
                ]),
            SubCommand::with_name("job")
                .about("Scheduling tasks for miner")
                .args(&[
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .default_value("all")
                        .help("scheduling tasks  for miner")
                ]),
        ])
}
