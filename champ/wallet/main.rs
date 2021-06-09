use clap::clap_app;

fn main() {
  let matches = clap_app!("champ-wallet" =>
      (version: "0.0.1")
      (author: "The POG Project <mail@henrygressmann.de>")
      (about: "POG's reference implementation in rust")
      (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
  )
  .get_matches();

  if let Some(c) = matches.value_of("config") {
    println!("Value for config: {}", c);
  }
}
