# esp-idf-captive-portal

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
