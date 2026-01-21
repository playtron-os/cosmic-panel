use anyhow::Result;
use cosmic_notifications_util::PANEL_NOTIFICATIONS_FD;
use smithay::reexports::rustix::{
    io::{FdFlags, fcntl_getfd, fcntl_setfd},
    {self},
};
use std::os::{
    fd::{AsRawFd, RawFd},
    unix::net::UnixStream,
};
use tracing::{info, error, warn};
use zbus::{connection::Builder, proxy};

#[proxy(
    default_service = "com.system76.NotificationsSocket",
    interface = "com.system76.NotificationsSocket",
    default_path = "/com/system76/NotificationsSocket"
)]
pub trait NotificationsSocket {
    /// get an fd for an applet
    fn get_fd(&self) -> zbus::Result<zbus::zvariant::OwnedFd>;
}

/// Connect to the notifications daemon with retries.
/// 
/// This function retries the connection because cosmic-notifications needs time
/// to set up its zbus server after being spawned. Without retries, panel may
/// try to connect before notifications is ready.
pub async fn notifications_conn() -> Result<NotificationsSocketProxy<'static>> {
    const MAX_RETRIES: u32 = 10;
    const RETRY_DELAY_MS: u64 = 500;
    
    let mut last_error = None;
    
    for attempt in 1..=MAX_RETRIES {
        
        match notifications_conn_inner().await {
            Ok(proxy) => {
                return Ok(proxy);
            }
            Err(e) => {
                warn!("notifications_conn attempt {} failed: {}", attempt, e);
                last_error = Some(e);
                
                if attempt < MAX_RETRIES {
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to connect after {} attempts", MAX_RETRIES)))
}

async fn notifications_conn_inner() -> Result<NotificationsSocketProxy<'static>> {
    info!("Connecting to notifications daemon");
    let fd_num = std::env::var(PANEL_NOTIFICATIONS_FD)
        .map_err(|_| anyhow::anyhow!("No {} env var found", PANEL_NOTIFICATIONS_FD))?;
    
    let raw_fd = fd_num
        .parse::<RawFd>()
        .map_err(|_| anyhow::anyhow!("Invalid {} env var", PANEL_NOTIFICATIONS_FD))?;

    // Additional validation - check FD with libc directly
    let libc_check = unsafe { libc::fcntl(raw_fd, libc::F_GETFD) };
    if libc_check == -1 {
        let err = std::io::Error::last_os_error();
        return Err(anyhow::anyhow!("FD {} is invalid: {}", raw_fd, err));
    }

    // Clone the FD instead of taking ownership, so retries can work
    let borrowed = unsafe { rustix::fd::BorrowedFd::borrow_raw(raw_fd) };
    let fd = borrowed.try_clone_to_owned()
        .map_err(|e| anyhow::anyhow!("Failed to clone FD {}: {:?}", raw_fd, e))?;
    let cloned_fd = fd.as_raw_fd();

    let res = fcntl_getfd(&fd).and_then(|flags| fcntl_setfd(&fd, FdFlags::CLOEXEC.union(flags)));

    let daemon_stream = match res {
        // CLOEXEC worked and we can startup with session IPC
        Ok(_) => {
            info!("Successfully set CLOEXEC on FD");
            UnixStream::from(fd)
        },
        // CLOEXEC didn't work, something is wrong with the fd, just close it
        Err(err) => {
            error!("Failed to set CLOEXEC on FD: {:?}", err);
            return Err(err.into());
        },
    };
    daemon_stream.set_nonblocking(true)?;

    let stream = tokio::net::UnixStream::from_std(daemon_stream)?;    
    let conn = Builder::socket(stream).p2p().build().await?;
    info!("Made socket connection");
    let proxy = NotificationsSocketProxy::new(&conn).await?;
    info!("Connected to notifications");

    Ok(proxy)
}
