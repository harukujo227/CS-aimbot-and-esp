# deadlocked

simple cs2 aimbot and esp, for linux only.

[![Open Source CS2 Hacking](https://badgen.net/discord/members/eXjG4Ar9Sx)](https://discord.gg/eXjG4Ar9Sx)

## Features

### Aimbot

- Hotkey
- FOV
- Smooth
- Start bullet
- Targeting mode
- Visibility check (VPK parsing)
- Head only/whole body
- Flash check
- FOV circle

### ESP

- Hotkey
- Box
- Skeleton
- Health bar
- Armor bar
- Player name
- Weapon icon
- Player tags (helmet, defuser, bomb)
- Dropped weapons
- Bomb timer

### Triggerbot

- Activation mode
- Min/max delay
- Additional Duration
- Visibility check
- Flash check
- Scope check
- Velocity threshold
- Head only mode

### Standalone RCS

- Smoothing

### Per-Weapon Overrides

- Aimbot
- Triggerbot
- RCS

### Misc

- Sniper crosshair
- Bomb timer
- Spectator list

### Unsafe

> [!WARNING]
> These features write to game memory and carry ban risk.

- No flash (with max flash alpha)
- FOV changer
- No smoke
- Smoke color change

> [!CAUTION]
> VACNet 3.0 is better at detecting aimbot and wallhacks. **Do not** use aim lock. Play with a low FOV. Use visuals sparingly.

## Setup

### Linux (Generic)

```bash
sudo usermod -aG input $(whoami)
# Restart your machine (required)
git clone --recursive https://github.com/avitran0/deadlocked
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### NixOS

Add `"input"` to your user's `extraGroups` in `configuration.nix`:

```nix
users.users.yourname = {
  isNormalUser = true;
  extraGroups = [ "wheel" "input" ];
};
```

Then rebuild and reboot:

```bash
sudo nixos-rebuild switch
sudo reboot
```

After reboot:

```bash
git clone --recursive https://github.com/avitran0/deadlocked
cd deadlocked
direnv allow
cargo run --release
```

If maps parsing fails, run (it will use Source2Viewer provided by nix direnv instead of one in resources):

```bash
cargo run --release -- --local-s2v
```

Everything is configured in `flake.nix` and `nix/shell.nix`.

### Fedora Atomic

```bash
grep -E '^input:' /usr/lib/group | sudo tee -a /etc/group && sudo usermod -aG input $USER
# Restart your machine (required)
git clone --recursive https://github.com/avitran0/deadlocked
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Running

```bash
./run.sh
```

> [!NOTE]
> When running for the first time and on game updates,
> it will parse the map data for a fast visibility check.
> Let this run until you see all maps have been parsed.
> This will take a lot of resources, so it's best to let it run before joining a game.

## FAQ

### Where are my configs saved?

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

### How do I configure the radar?

See [radar.md](radar.md)

### Which desktop environments and window managers are supported?

**Best support:**

- GNOME (Mutter)
- KDE (KWin)

**Good support:**

- SwayWM
- Weston

**Fair support:**

- i3
- OpenBox
- XFCE

**Limited/No support:**

- Hyprland (poor X11 support)
- Other Wayland-only compositors

### I'm using Hyprland and something doesn't work

Hyprland has poor X11 support for the techniques this cheat uses. This cannot be fixed.

### I'm using Gamescope and the overlay is too small

The game still thinks it's running in 16:9 resolution. This cannot be fixed.

### My screen/overlay is black

Your compositor or window manager doesn't support transparency, or it's not enabled.

### The overlay shows but I can't click anything

The window couldn't be made click-through. This is a window manager/compositor limitation.

### The overlay doesn't show up

Your window manager doesn't support positioning or resizing windows.

### The overlay isn't on top of other windows

Your window manager doesn't support always-on-top windows.
