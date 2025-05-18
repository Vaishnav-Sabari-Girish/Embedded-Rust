# Pins that are excited 

1. LED D2 is connected to ROW1 COL1
2. ROW1 = P0.21 
3. COL1 = P0.28

For nRF52833, we do not need to enable peripheral clocks before trying to setup the GPIO pins. 

Configuration for GPIO pins happen in the PIN_CNF\[n\] registers , where n = 0, 1, 2, ...31. 

Base Address for GPIO is 0x50000000. 

This is also the Base address for Port 0. 

| Register | Offset | Description |
| --------------- | --------------- | --------------- |
| OUT | 0x504 | Write GPIO Port |
| IN | 0x510 | Read GPIO Port |
|PIN_CNF\[21\] | 0x754 | Pin Configuration for Pin 21|
|PIN_CNF\[28\] | 0x770 | Pin Configuration for Pin 28|


## PIN_CNF\[n\] Register Bit 

| Value | Description |
| -------------- | --------------- |
| 0 | Input |
| 1 | Output |


To toggle the LED, we toggle the PIN21 bit in the OUT register to switch ON and OFF the LED. 

# Output

![Video](./videos/bare_metal_blink.mp4)
