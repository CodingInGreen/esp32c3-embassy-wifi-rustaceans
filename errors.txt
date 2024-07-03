error[E0432]: unresolved import `esp_wifi::wifi_interface`
  --> src/main.rs:35:5
   |
35 |     wifi_interface::WifiStack,
   |     ^^^^^^^^^^^^^^ could not find `wifi_interface` in `esp_wifi`
   |
note: found an item that was configured out
  --> /Users/hott/.cargo/registry/src/index.crates.io-6f17d22bba15001f/esp-wifi-0.6.0/src/lib.rs:57:9
   |
57 | pub mod wifi_interface;
   |         ^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0432`.