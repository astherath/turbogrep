use std::fmt;
pub fn clap_panic<T: fmt::Display>(details: T) -> ! {
    clap::Error::with_description(
        &format!("Error processing command. Details: {}", details),
        clap::ErrorKind::InvalidValue,
    )
    .exit()
}
