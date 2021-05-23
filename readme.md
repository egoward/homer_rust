## Description
This is a prototype Rust version of the super hacky [homer_node](https://github.com/egoward/homer-node) NodeJS project.  It's primarily intended to teach me Rust, and also learn about how it can be practically used in a real embedded / IoT systems.

The project is intended to run on devices like Raspberry PI and  ...

Read from:
- [1-write sensors](https://tutorials-raspberrypi.com/raspberry-pi-temperature-sensor-1wire-ds18b20/)
- Bluetooth LE sensors, currently using [btleplug](https://github.com/deviceplug/btleplug)

Publish to:
- MQTT, supporting [Home Assistant MQTT Discovery](https://www.home-assistant.io/docs/mqtt/discovery/) 
- [AWS CloudWatch](https://aws.amazon.com/cloudwatch/)


## Why does it do it?
Home automation tasks such as:
- Automatically Open velux window when bathroom is hot and humid
- Trigger a CloudWatch alarm when fish tank is too cold


## Current State

The code as is will only compile on Windows due to some issues with the Bluetooth driver abstraction layer.  This will be resolved when I understand Rust better.

Pi 1-Wire support is missing.  It should trivial as it's just presented as a filesystem.

Dependencies are a bit of a mess with both MQTT driver Bluetooth requiring a worker thread in addition to [Tokio](https://tokio.rs/) dependenices.  It looks like the Tokio archiecture could scale to microcontrollers but it's not there yet.

## Next steps (maybe)

- Make bluetooth stack portable compile on PI and Mac.
- Actually implement 1-Wire on PI
- Minimize dependencies
- Understand libraries, dynamic loading and ABI's