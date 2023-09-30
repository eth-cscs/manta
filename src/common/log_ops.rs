use std::str::FromStr;

use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

// Code base log4rs configuration to avoid having a separate file for this to keep portability
pub fn configure(log_level: String) {
    let audit_file_path = "/var/log/manta/requests.log";

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {h({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build();

    let requests_rslt = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build(audit_file_path);

    let mut config_builder = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(
            Logger::builder()
                .appender("stdout")
                .build("app::backend", LevelFilter::Info),
        );

    // Configure logs for audit file
    let error_configuring_audit_logs;
    config_builder = match requests_rslt {
        Ok(requests) => {
            config_builder = config_builder
                .appender(Appender::builder().build("requests", Box::new(requests)))
                .logger(
                    Logger::builder()
                        .appender("requests")
                        .additive(false)
                        .build("app::audit", LevelFilter::Info),
                );

            error_configuring_audit_logs = None;

            config_builder
        }
        Err(error) => {
            error_configuring_audit_logs = Some(error);
            config_builder
        }
    };

    let config = config_builder
        .build(
            Root::builder()
                .appender("stdout")
                .build(LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Error)),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    // use handle to change logger configuration at runtime

    if error_configuring_audit_logs.is_some() {
        log::warn!(
            "Unable to create audit file {}. Reason: {:?}. Continue",
            audit_file_path,
            error_configuring_audit_logs
        );
    }
}
