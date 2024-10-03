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





