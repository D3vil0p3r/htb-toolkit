# HTB Toolkit

![image](https://github.com/D3vil0p3r/htb-toolkit/assets/83867734/1455a5db-fa91-485b-91ba-bb27675357b9)

**HTB Toolkit** allows you to play Hack The Box machines directly on your system.

# Usage

To use HTB Toolkit, you need to retrieve an **App Token** from your Hack The Box [Profile Settings](https://app.hackthebox.com/profile/settings) and click on **Create App Token** button under **App Tokens** section.

Once generated and copied on clipboard the App Token, on the terminal run:
```
htb-toolkit -k set
```
and, after **Password:** prompt, paste the App Token value and press **Enter**. It will be stored in a secure manner.

**Don't share your App Token with anyone!**

Showcase of HTB Toolkit:

[![HTB Toolkit Asciicast](https://github.com/D3vil0p3r/htb-toolkit/assets/83867734/cfc8aac4-f58e-4b44-8ac1-12e1842c801f)](https://asciinema.org/a/605148)
Source: [Asciinema](https://asciinema.org/a/605148)

# Install

## Arch-based Linux distro
Add Athena OS repository to your system as described [here](https://athenaos.org/en/configuration/repositories/#installation).

Run:
```
sudo pacman -Syyu
sudo pacman -S htb-toolkit
```

# Build from source
## Non-Arch-based Linux distro
Install the following runtime dependencies:
```
coreutils gnome-keyring gzip imagemagick openvpn
```
Install the following build dependencies:
```
git cargo
```
Clone the repository by:
```
git clone https://github.com/D3vil0p3r/htb-toolkit
cd htb-toolkit
cargo build --release
```
It will create the binary file **htb-toolkit** in `htb-toolkit/target/release`. Copy this file to a binary folder as:
```
sudo cp htb-toolkit/target/release/htb-toolkit /usr/bin/
```
