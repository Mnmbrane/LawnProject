#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

#[path = "main/wifi.rs"]
mod wifi;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::timer::timg::TimerGroup;

use log::info;

use embassy_executor::Spawner;

use esp_backtrace as _;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.2.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73726);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    // TODO: Spawn some tasks
    let _ = spawner;

    let ssid = option_env!("WIFI_SSID").expect("WIFI_SSID not set");
    let pass = option_env!("WIFI_PASS").expect("WIFI_PASS not set");

    info!("{ssid}");
    wifi::configure(&mut wifi_controller, ssid, pass).expect("Failed to configure Wi-Fi");
    info!("Wi-Fi configured");

    info!(
        "Wi-Fi started before explicit start: {}",
        wifi_controller.is_started().unwrap_or(false)
    );
    wifi_controller
        .start_async()
        .await
        .expect("Failed to start Wi-Fi controller");
    info!(
        "Wi-Fi started after explicit start: {}",
        wifi_controller.is_started().unwrap_or(false)
    );

    match wifi::scan_visible(&mut wifi_controller).await {
        Ok(aps) => {
            info!("Visible AP count: {}", aps.len());
            let mut found_target = false;
            for ap in aps {
                if ap.ssid == ssid {
                    found_target = true;
                }
                info!(
                    "Visible AP: ssid={}, channel={}, rssi={}, auth={:?}",
                    ap.ssid, ap.channel, ap.signal_strength, ap.auth_method
                );
            }

            if !found_target {
                info!("Target AP not found during scan for ssid={ssid}");
            }
        }
        Err(err) => info!("Visible AP scan failed: {:?}", err),
    }

    for attempt in 1..=5 {
        info!("Wi-Fi attempt {} starting", attempt);
        match wifi::connect(&mut wifi_controller).await {
            Ok(()) => {
                if wifi::is_connected(&wifi_controller) {
                    info!("Wi-Fi connected on attempt {}", attempt);
                    break;
                } else {
                    info!(
                        "Wi-Fi connect returned OK on attempt {}, but link is not up yet",
                        attempt
                    );
                }
            }
            Err(err) => {
                info!("Wi-Fi attempt {} failed: {:?}", attempt, err);
            }
        }

        if attempt < 5 {
            info!("waiting 2s before retry");
            delay.delay_millis(2_000);
        } else {
            info!("Wi-Fi failed after 5 attempts");
        }
    }

    loop {
        info!("Hello world! By Manny");
        delay.delay_millis(1_000);
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
