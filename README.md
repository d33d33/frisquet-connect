# frisquet-connect

## Usage

### Upload an rf sketch

A sketch for a feather M0, is available under `/arduino`
A sketch for a heltech lora v3 is available under `/heltech-frisquet-serial`

### Create a `config.toml` config file

For serial:

``` toml
[serial]
port = "/dev/cu.usbmodem111201"
speed =
```

For mqtt:

``` toml
[mqtt]
broker = "tcp://localhost:1883"
client_id = "frisquet-connect"
cmd_topic = "/frisquet/cmd"
lst_topic = "/frisquet/listen"
```

### Run pair

Put the boiler in pairing mode

``` bash
cargo run -- pair 
```

### Get sensors

``` bash
cargo run -- sensors 
```

### Set exteriror temperature

``` bash
cargo run -- sonde 12.4 
```

### Configure area 1

Update `area1` config in your `config.toml` (see template).
The boiler may take a while to process this command (up to 5min).
`frisquet-connect` will retry if needed.

``` bash
cargo run -- area1 
```

### Get available commands

``` bash
cargo run -- help 
```
