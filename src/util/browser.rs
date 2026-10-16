#[cfg(feature = "webbrowser")]
pub use webbrowser::open;

// stub for platforms webbrowser doesn't support, such as Horizon OS
#[cfg(not(any(feature = "webbrowser", target_os = "horizon", target_os = "ios")))]
pub fn open(_url: &str) -> std::io::Result<()> {
    Ok(())
}

#[cfg(all(target_os = "ios", not(feature = "webbrowser")))]
pub fn open(url: &str) -> std::io::Result<()> {
    crate::util::ios::open_url(url);
    Ok(())
}

#[cfg(target_os = "horizon")]
pub fn open(url: &str) -> std::io::Result<()> {
    crate::framework::backend_horizon::web_open(url)
}
