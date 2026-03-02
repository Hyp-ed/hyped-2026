use embassy_time::{Duration, Timer};
use hyped_communications::bus::EVENT_BUS;
use hyped_communications::events::Event;
use hyped_core::types::{Current, Velocity, Temperature, Voltage};

#[embassy_executor::task]
pub async fn propulsion_status_task() -> ! {
    let tx = EVENT_BUS.sender();

    loop {
        // TODO: Replace with real readings from motor controller / sensors
        let current_ma = Current(0);
        let velocity_kmh = Velocity(0);
        let temperature_c = Temperature(0);
        let voltage_cv = Voltage(0);

        tx.send(Event::PropulsionStatus {
            current_ma,
            velocity_kmh,
            temperature_c,
            voltage_cv,
        })
        .await;

        Timer::after(Duration::from_millis(100)).await; // pick a sensible rate (e.g. 10 Hz)
    }
}