use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("kvs")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").index(1).required(true))
                .arg(Arg::with_name("value").index(2).required(true)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("key").index(1).required(true)))
        .get_matches();

    let mut store = kvs::KvStore::new();

    if let Some(m) = matches.subcommand_matches("set") {
        let key = m.value_of("key").unwrap().into();
        let value = m.value_of("value").unwrap().into();

        store.set(key, value);
    } else if let Some(m) = matches.subcommand_matches("get") {
        let key = m.value_of("key").unwrap().into();

        store.get(key);
    }
}
