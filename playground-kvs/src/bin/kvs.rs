use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, AppSettings, Arg,
    SubCommand,
};

fn main() {
    let app = app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("set")
                .arg(Arg::with_name("key").index(1).required(true))
                .arg(Arg::with_name("value").index(2).required(true)),
        )
        .subcommand(SubCommand::with_name("get").arg(Arg::with_name("key").index(1).required(true)))
        .subcommand(SubCommand::with_name("rm").arg(Arg::with_name("key").index(1).required(true)));
    let matches = app.get_matches();

    let mut store = kvs::KvStore::new();

    if let Some(m) = matches.subcommand_matches("set") {
        let key = m.value_of("key").unwrap().into();
        let value = m.value_of("value").unwrap().into();

        store.set(key, value);
        panic!("{}", "unimplemented");
    } else if let Some(m) = matches.subcommand_matches("get") {
        let key = m.value_of("key").unwrap().into();

        store.get(key);
        panic!("{}", "unimplemented");
    } else if let Some(m) = matches.subcommand_matches("rm") {
        let key = m.value_of("key").unwrap().into();

        store.remove(key);
        panic!("{}", "unimplemented");
    }
}
