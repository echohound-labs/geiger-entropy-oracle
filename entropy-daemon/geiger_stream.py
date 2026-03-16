import serial
import time
import hashlib

PORT = "/dev/ttyUSB0"
BAUD = 115200

ser = serial.Serial(PORT, BAUD, timeout=1)

last_event_time = None
last_cps = 0

print("Geiger entropy collector started...")

while True:

    # --- GET CPS ---
    ser.write(b"<GETCPS>>")
    cps_data = ser.read(4)

    if len(cps_data) != 4:
        continue

    cps = int.from_bytes(cps_data, "big")

    # --- GET CPM ---
    ser.write(b"<GETCPM>>")
    cpm_data = ser.read(4)

    if len(cpm_data) != 4:
        continue

    cpm = int.from_bytes(cpm_data, "big")

    # approximate dose
    usv = cpm * 0.0065

    now = time.time()

    # detect new decay event
    if cps >= 1 and last_cps == 0:

        if last_event_time is not None:

            delta = now - last_event_time

            entropy = hashlib.sha256(
                f"{delta}-{now}-{cpm}-{cps}".encode()
            ).hexdigest()

            print(
                f"DECAY EVENT | Δt={delta:.3f}s | CPM={cpm} | µSv/h={usv:.3f} | entropy={entropy[:16]}"
            )

        last_event_time = now

    last_cps = cps

    time.sleep(0.25)
