# Motor Control Board Interface

## NAV → MotorControl CAN
Extended IDs, split by function to keep each payload ≤8 bytes:

- `0x18FF0301` — Commands (first byte is opcode)
  - `0x01` StartDrive
  - `0x02` Shutdown
  - `0x03` QuickStop
  - `0x04` SwitchOn
  - `0x05` SetFrequency  
    - bytes 1..4: `u32` frequency (Hz), little‑endian
- `0x18FF0302` — Kinematics: speed + accel (no opcode byte)  
  - bytes 0..3: `i32` speed in millimetres per second, little‑endian  
  - bytes 4..7: `i32` acceleration in millimetres per second², little‑endian  
  - Board converts to `f32` m/s and m/s² by dividing by 1000.0.
- `0x18FF0303` — Kinematics: position + target (no opcode byte)  
  - bytes 0..3: `i32` position in millimetres, little‑endian  
  - bytes 4..7: `i32` target position in millimetres, little‑endian  
  - Board converts to `f32` metres by dividing by 1000.0.

Notes:
- Endianness: all kinematic fields are little‑endian.
- Filters: CAN1 should accept all three IDs into the NAV FIFO.

## Braking Logic
- Collects kinematics from `0x18FF0302` and `0x18FF0303`.
- Computes stopping distance: `v²/(2·a_max) + margin`, using placeholders:
  - `POD_MASS_KG=200.0`, `MAX_BRAKE_FORCE_N=5000.0`, `BRAKE_MARGIN_M=5.0` (replace with real values).
- When within stopping distance and moving: sends CANOpen `QuickStop` + `Shutdown` to EMSISO, then engages mechanical brake solenoid (GPIO driven low, TODO replace placeholder pin `PC13`).
- Overshoot (distance≤0): engages brakes and issues `QuickStop`.
- Brakes stay engaged once applied (no auto-release).
- Emergency signal sets a `FORCE_BRAKE` flag: immediately issues `QuickStop` and clamps the brake solenoid even if kinematic data is missing.

## Hardware Assumptions / TODOs
- Brake solenoid GPIO: currently `PC13`, active-low; replace with the actual pin/polarity.
- Optionally add ToF actuation verification via `Pneumatics` once I2C bus/pins are known.
