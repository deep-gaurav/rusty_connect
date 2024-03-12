use api::SendBattery;
use battery::units::ratio::percent;
use tracing::debug;

use crate::{GQL_PORT, REQWEST_CLIENT};

pub async fn send_batery() -> anyhow::Result<()> {
    loop {
        let data = {
            let battery_manager = battery::Manager::new()?;
            let mut batteries = battery_manager.batteries()?;
            if let Some(Ok(battery)) = batteries.next() {
                let charge = battery.state_of_charge().get::<percent>();
                let is_charging = battery.time_to_full().is_some();

                Some((charge, is_charging))
            } else {
                None
            }
        };
        if let Some((charge, is_charging)) = data {
            let response = graphql_client::reqwest::post_graphql::<SendBattery, _>(
                &REQWEST_CLIENT,
                format!("http://localhost:{GQL_PORT}"),
                api::send_battery::Variables {
                    device_id: None,
                    is_charging,
                    charge: charge.into(),
                },
            )
            .await;
            if let Err(err) = response {
                debug!("Failed to send battery info {err:?}")
            }
        } else {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
