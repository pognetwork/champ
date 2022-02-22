use clap::{self, Arg};

pub fn new() -> clap::ArgMatches {
    clap::Command::new("champ-node")
        .version("0.0.1")
        .author("The POG Project <contact@pog.network>")
        .about("POGs reference implementation in rust")
        .arg(Arg::new("web").long("feat-web").takes_value(false).help("enables web interface"))
        .arg(Arg::new("metrics").long("feat-metrics").takes_value(false).help("enables metrics api"))
        .arg(Arg::new("roughtime").long("feat-roughtime").takes_value(false).help("enables roughtime server"))
        .arg(
            Arg::new("loglevel")
                .short('l')
                .long("loglevel")
                .value_name("LOGLEVEL")
                .help("Sets a log level. Can be one of `trace`, `debug`, `info`, `warn`, `error` ")
                .takes_value(true),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .subcommand(
            clap::Command::new("admin")
                .about("access to the admin interface")
                .subcommand(
                    clap::Command::new("create-user")
                        .about("creates a user for the web api")
                        .after_help("Format: -u [username] -p [password]")
                        .arg(
                            Arg::new("username")
                                .short('u')
                                .help("new username")
                                .takes_value(true)
                                .value_name("USERNAME")
                                .forbid_empty_values(true),
                        )
                        .arg(
                            Arg::new("password")
                                .short('p')
                                .help("new password")
                                .takes_value(true)
                                .value_name("PASSWORD")
                                .forbid_empty_values(true),
                        )
                        .arg(
                            Arg::new("perms")
                                .help("adds permissions")
                                .takes_value(true)
                                .multiple_values(true)
                                .value_name("PERMISSIONS")
                                .forbid_empty_values(false)
                                .max_values(20)
                                .min_values(0),
                        ),
                )
                .subcommand(clap::Command::new("generate-key").about("generates a node private key used for JWTs")),
        )
        .get_matches()
}
