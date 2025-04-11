# sonar
- The better ping (maybe?)

## Usage
### Using my repo (For Arch-based distros)
```sh
# Install pacsync command
sudo pacman -S --needed pacutils

# Add repo
echo -e "\n\n[landware]\nSigLevel = Optional TrustAll\nServer = https://kage.sj.strangled.net/landware" | sudo tee -a /etc/pacman.conf

# Sync repo without syncing all repos
sudo pacsync landware

# Install like a normal package
sudo pacman -S sonar-git
```

### Building
```sh
# Install deps
# Arch Linux
pacman -S rust

# Clone the repo
git clone https://codeberg.org/Land/sonar.git
cd sonar

# Build
cargo b --release
```

### Options
| **Argument**            | **Description**                                  |
|----------------------------|-----------------------------------------------|
| `-v, --verbose`             | increase verbosity                           |
| `-V, --version`             | Outputs version.                             |
| `-c, --count`               | amount to attempt pinging                    |
| `-i, --interval`            | seconds to wait between sending packets      |
| `-x, --extra`               | enable querying for extra information        |
| `IP`                        | ip address to ping                           |