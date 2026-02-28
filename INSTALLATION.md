# Build from source

## Copy-paste ready

```bash
git clone https://github.com/TornaxO7/vibe
cd vibe
cargo run --release
```

## Step-by-step

1. Install the dependencies specified below (if your package manager isn't listed here, feel free to create a PR!):

### `apt (Debian/Ubuntu-based distributions)`

```bash
sudo apt install rustup librust-wayland-client-dev librust-alsa-sys-dev libxkbcommon-dev pavucontrol
```

### `dnf (Fedora-based distributions)`

```bash
sudo dnf install rustup rust-wayland-client-devel rust-alsa-sys-devel libxkbcommon-devel pavucontrol
```

### `pacman (Arch-based distributions)`

```bash
sudo pacman -S rustup rust-wayland-client-devel rust-alsa-sys-devel libxkbcommon-devel pavucontrol
```

2. Clone the repository:

```bash
git clone https://github.com/TornaxO7/vibe
```

3. Enter the `vibe` directory:

```bash
cd vibe
```

4. (optional) Set the rust toolchain:

```bash
rustup default stable
```

5. Compile and run the binary:

```bash
cargo run --release
```

6. (optional) Install the binary:

```bash
cargo install --release
```

7. Your microphone is very likely be caught as the audio source.
   To fix that:
   1. start the application `pavucontrol`.
   2. At the top: Click on `Recording`
   3. There should be an entry (something like `ALSA[vibe]`). On the right, click on the drop down menu.
   4. Select the audio source (often "Monitor _bla_ Built-in Audio _bla_")
8. (optional) [Configure](https://github.com/TornaxO7/vibe/wiki/Config) `vibe`!

# Installation with supported package managers

Here's a list of package manager commands which you can copy+paste to install the required dependencies. If your package manager isn't listed here, feel free to create a PR!

### `AUR` (Arch)
Install from the AUR with your favourite AUR-helper. See the ArchWiki to learn more.

```bash
paru -S vibe-audio-visualizer-git
```

### `flakes` (Nix)

You can simply run the binary with the following command:

```bash
nix run github:TornaxO7/vibe
```

This flake also provides a package for it if you'd like to install it permanently. Take a look into the flake.nix for that.
