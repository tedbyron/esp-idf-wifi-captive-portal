# esp-idf-wifi-captive-portal

Example ESP-IDF captive portal for setting up STA WiFi

- WSL

  ```ps1
  usbipd list
  usbipd attach -awb <BUSID>
  ```

- \*nix

  ```sh
  cargo r --release
  ```

Uses websockets to communicate from the browser to the ESP device (`CONFIG_HTTPD_WS_SUPPORT=y` in
sdkconfig.defaults) and the crate `edge-captive` for the DNS captive portal.
