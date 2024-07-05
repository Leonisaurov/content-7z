# Content 7z
Tool for visualize the content of a 7z file and navigate around the files, all with a friendly TUI.

[![A look of the tool](https://asciinema.org/a/666845.svg)](https://asciinema.org/a/666845)

## Index
1. [Dependencies](#dependencies)
    - [No which](#which)
2. [Usage](#usage)
3. [Installation](#install)
4. [Configurations](#config-file)
5. [Contribute](contribute)

## Dependencies
It has 2 important dependencies, rust and the 7z terminal executable.

To download 7z just get it from your package manager, for example:
- Arch like:
```bash
pacman -S p7zip
```
- Ubuntu like:
```bash
apt install p7zip
```
- Termux:
```bash
pkg install p7zip
```

It is similar to get rust just follow their [official guide](https://www.rust-lang.org/es/tools/install) by downloading rustup. Or you can try with your package manager:
- Arch like:
```bash
pacman -S rust
```
- Ubuntu like:
```bash
apt install rust
```
- Termux:
```bash
pkg install rust
```

In the case of termux, it is currently not possible to use the official rustup binary, so it is required to use the package manager.

### Which
<details>
<summary>NO_WHICH</summary>

Another not very relevant dependency is which, it helps the installation file to identify that the other dependencies are present. It is usually installed in most distributions by default, if not, you can use your package manager to do it:
- Arch like:
```bash
pacman -S which
```
- Ubuntu like:
```bash
apt install which
```
- Termux:
```bash
pkg install which
```

If you do not have "which" installed, and do not want to install it, having the dependencies mentioned above, just include the environment variable NO_WHICH=ACTIVE, during the installation, this will cause the compile file to not use which, therefore, it will not check for the presence of the other dependencies.
```bash
NO_WHICH=ACTIVE BIN="$PREFIX/bin" ./install
```
</details>

## Install
Clone this repository with git and get into the source code directory:
```bash
git clone 'https://github.com/Leonisaurov/content-7z' content-7z
cd content-7z
```

Give run permissions to the file "install":
```bash
chmod +x ./install
```

And execute:
```bash
./install
```
It will compile the program through cargo and if you don't have a configuration file already created, it will create one by default.

The executable will be found in target/release/content-7z, if you want to install it in a more convenient location, set the environment variable BIN to the location where you want the program to be.
```bash
BIN=/usr/local/bin ./install
```

In termux it would be:
```bash
BIN=$PREFIX/bin ./install
```

That's it, now you have content-7z on your system

## Usage
Type content-7z and a name of a compressed file:
```bash
content-7z any.7z
```

You can move around with 2 keys:
1. Enter, to move into the folder.
2. Backspace, to go back to the parent folder.

The mouse can be move with the arrow keys.
If you want to exit, just press Escape or 'q'.

To open a file, press 'o' while your cursor is over the file to open, it will open in the most specific editor it finds:
- If there is one defined in the configuration file, it will use that one.
- If there is not one defined in the configuration file, it will look to see if the environment variable "EDITOR" is defined, if it is, it will use that editor.
- If none of the above works, it will run the 'editor' binary.

You can try 'content-7z' in the compress_examples folder, with compressed files in different formats.
```bash
cd ./compressed_examples
content-7z any.7z
```

## Config File
The configuration file, using the toml format, is called "content-7z.toml", and is located in "$HOME/.config/content-7z.toml".
A basic configuration file will be created automatically when installing the program.
```toml
# Catppuccine
# background-color = [30, 30, 46]
# text-color = [205, 214, 244]
# border-color = [69, 71, 90]

# Tokio Dark
# background-color = [17, 18, 29]
# text-color = [160, 168, 205]
# border-color = [74, 80, 87]

# Rose pine
# text-color = [224, 222, 244]
# background-color = [25, 23, 36]
# border-color = [110, 106, 134]

# Github Default
# text-color = [230, 237, 243]
# background-color = [13, 17, 23]
# border-color = [110, 118, 129]

# Lunaperche
# text-color = [198, 198, 198]
# background-color = []
# border-color = [88, 88, 88]

# Fancy colored icons
# folder-bullet = " \ue5fe "
# folder-bullet-color = "1;34"

# file-bullet = " \uea7b "
# file-bullet-color = "38;2;88;88;88;1"

# editor = ""
```

You can uncomment some lines of the configuration file to set the properties and test the themes or create your own, it is advisable to uncomment and assign the editor configuration line, "editor", so that content-7z can identify which editor to use specifically.

It has 8 customizable properties, which can be text strings, or rgb colors (defined as a list of 3 numbers) depending on which property it is:
- background-color (default: [0, 0, 0, 0])
- text-color (default: [200, 200, 200])
- border-color (default: [255, 255, 255, 255])
- folder-bullet (default: "[+]")
- folder-bullet-color (default: [200, 200, 200])
- file-bullet (default: "--- ")
- file-bullet-color (default: [200, 200, 200])
- editor to use (default: "")

Colors can be defined in 2 ways:
1. RGB: a list of 3 numbers representing red, green and blue,  ranging from:
```toml
*-color = [0, 0, 0, 0]
```
to:
```toml
*-color = [255, 255, 255]
```
2. Ansi codes: you can also use a text, which has the central part of an ansi code to change colors, e.g. To use the blue color of the terminal, but in bold:
```toml
*-color = "1;34"
```

## Contribute
The repository is completely open to contributions, just make a pull request and as far as possible, I will try to see them and accept them.
