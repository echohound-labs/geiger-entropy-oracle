import serial

ser = serial.Serial("/dev/ttyUSB0",115200,timeout=2)

# GETVER command (binary format)
cmd = b"<GETVER>>"
ser.write(cmd)

data = ser.read(64)

print("RAW:", data)
