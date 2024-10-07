# Croptesting knyst

Contains test code to debug Mult results on stereo wav file.

## Unprocessed output
When running:

```shell
cargo run -- --volume 1.0
```
No processing is done. Both channels of the wav are outputted directly. The balance is perfectly in the center.

## Output passed through Mult
When running:

```shell
cargo run -- --volume 0.999
```
Both channels of the wav are passed through a `Mult` node where they are multiplied by the volume. The balance is slightly shifted to the right ear.

The left channel is `delayed` by `buffer_size`. When setting the buffer size to sample rate, the left channel is delayed by 1 second.

When using the alternative multiplier `Multiplier`, this effect does not occur.

Test commands that generate audio files:

```shell
cargo run -- --render-type tone
cargo run -- --render-type tone --volume 1.00001 --multiplier multiplier
cargo run -- --render-type tone --volume 1.00001 --multiplier mult

cargo run -- --render-type file
cargo run -- --render-type file --volume 1.00001 --multiplier multiplier
cargo run -- --render-type file --volume 1.00001 --multiplier mult
```