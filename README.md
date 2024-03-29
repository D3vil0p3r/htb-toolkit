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
Interactive source: [Asciinema](https://asciinema.org/a/605148)

# Install

## Arch-based Linux distro
Add [Athena OS](https://athenaos.org/) repository to your system as described [here](https://athenaos.org/en/configuration/repositories/#installation).

Run:
```
sudo pacman -Syyu
sudo pacman -S htb-toolkit
```

# Build from source
## Non-Arch-based Linux distro
Install the following runtime dependencies:

**Arch-based distros**
```
coreutils gnome-keyring gzip libsecret noto-fonts-emoji openssl openvpn ttf-nerd-fonts-symbols
```
**Debian-based distros**
```
coreutils fonts-noto-color-emoji gnome-keyring gzip libsecret-tools libssl-dev openvpn

wget https://github.com/ryanoasis/nerd-fonts/releases/latest/download/NerdFontsSymbolsOnly.zip
unzip NerdFontsSymbolsOnly.zip -x LICENSE readme.md -d ~/.fonts
fc-cache -fv
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
Now you can run:
```
htb-toolkit -h
```

# FlyPie Integration in Athena OS

HTB Toolkit can be integrated in FlyPie menu of Athena OS by `htb-toolkit -u` command. It will implement **shell-rocket** as terminal wrapper inside the FlyPie menu HTB machine icons to run HTB machines.
