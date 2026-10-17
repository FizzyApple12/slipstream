use std::time::SystemTime;

#[cfg(all(debug_assertions, not(feature = "debug_log")))]
pub fn setup_logger(log_path: Option<String>) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(
            log_path.unwrap_or("ddj_flx10.log".to_string()),
        )?)
        .apply()?;

    Ok(())
}

#[cfg(all(debug_assertions, feature = "debug_log"))]
pub fn setup_logger(log_path: Option<String>) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ));
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(
            log_path.unwrap_or("ddj_flx10.log".to_string()),
        )?)
        .apply()?;

    Ok(())
}

#[cfg(not(debug_assertions))]
pub fn setup_logger(log_path: Option<String>) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(
            log_path.unwrap_or("ddj_flx10.log".to_string()),
        )?)
        .apply()?;

    Ok(())
}
