use std::{env, panic, process};

use backtrace::Backtrace;
use structopt::StructOpt;

use road_render::cli::Options;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        // PanicInfo's payload is usually a &'static str or String
        let message = match panic_info.payload().downcast_ref::<&str>() {
            Some(&message) => message.to_string(),
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(message) => message.clone(),
                None => "<no message>".to_string(),
            },
        };
        log::error!("RoadRender crashed!");
        log::error!("This is probably a bug.");
        log::error!("");
        log::error!(
            "Please consider filing an issue: {}/issues",
            env!("CARGO_PKG_REPOSITORY")
        );
        log::error!("");
        log::error!("Details: {}", message);

        if let Some(location) = panic_info.location() {
            log::error!("in file {} on line {}", location.file(), location.line());
        }

        // When using the backtrace crate, we need to check the RUST_BACKTRACE
        // environment variable ourselves. Once we switch to the (currently
        // unstable) std::backtrace module, we won't need to do this anymore.
        let should_backtrace = env::var("RUST_BACKTRACE")
            .map(|var| var == "1")
            .unwrap_or(false);

        if should_backtrace {
            eprintln!("{:?}", Backtrace::new());
        } else {
            eprintln!(
                "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace."
            );
        }

        process::exit(1);
    }));

    // env_logger::Builder::from_env(env_logger::Env::default())
    //     .format_module_path(false)
    //     .format_timestamp(None)
    //     // Indent following lines equal to the log level label, like `[ERROR] `
    //     .format_indent(Some(8))
    //     .init();

    let options = Options::from_args();

    let log_filter = match options.global.verbosity {
        0 => "info",
        1 => "info,librojo=debug",
        2 => "info,librojo=trace",
        _ => "trace",
    };

    let log_env = env_logger::Env::default().default_filter_or(log_filter);

    env_logger::Builder::from_env(log_env)
        .format_module_path(false)
        .format_timestamp(None)
        // Indent following lines equal to the log level label, like `[ERROR] `
        .format_indent(Some(8))
        .write_style(options.global.color.into())
        .init();

    if let Err(err) = options.run() {
        log::error!("{:?}", err);
        process::exit(1);
    }
}
