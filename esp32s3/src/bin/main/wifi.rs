use core::result::Result;
use esp_hal::delay::Delay;
use esp_radio::wifi::{
    AccessPointInfo, AuthMethod, ClientConfig, ModeConfig, ScanConfig, WifiController, WifiError,
};

pub async fn scan_visible(
    controller: &mut WifiController<'_>,
) -> Result<alloc::vec::Vec<AccessPointInfo>, WifiError> {
    controller.scan_with_config(ScanConfig::default().with_show_hidden(true).with_max(64))
}

pub fn configure(
    controller: &mut WifiController<'_>,
    ssid: &str,
    password: &str,
) -> Result<(), WifiError> {
    let mut client = ClientConfig::default()
        .with_ssid(ssid.into())
        .with_password(password.into());

    client = if password.is_empty() {
        client.with_auth_method(AuthMethod::None)
    } else {
        client.with_auth_method(AuthMethod::WpaWpa2Personal)
    };

    controller.set_config(&ModeConfig::Client(client))
}

pub async fn connect(controller: &mut WifiController<'_>) -> Result<(), WifiError> {
    let delay = Delay::new();

    if !matches!(controller.is_connected(), Ok(true)) {
        let _ = controller.disconnect();
        controller.connect()?;

        for _ in 0..40 {
            match controller.is_connected() {
                Ok(true) => return Ok(()),
                Err(WifiError::Disconnected) => return Err(WifiError::Disconnected),
                Ok(false) => delay.delay_millis(250),
                Err(err) => return Err(err),
            }
        }

        return Err(WifiError::Disconnected);
    }

    Ok(())
}

pub fn is_connected(controller: &WifiController<'_>) -> bool {
    matches!(controller.is_connected(), Ok(true))
}
