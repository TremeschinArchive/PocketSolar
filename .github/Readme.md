👆【☰】Table of Contents

<div align="center">
    <img src="https://github.com/BrokenSource/ViyLine/raw/Master/Website/icon.png" onerror="this.src='../Website/icon.png'" width="256"/>

    ViyLine
</div>

<h3>Solar panel IV curve tracker using a PIC16F877A Microcontroller, HC-06 Bluetooth module and a front end GUI in Rust.</h3>

*This is a VERY DIY project, you must build the physical circuit, we provide you the schematic and list of components.*

<br>



# ● Running from Source Code

1. Download [The Source Code](https://github.com/BrokenSource/ViyLine/archive/refs/heads/Master.zip) of ViyLine or clone this repo with:
    - `git clone https://github.com/BrokenSource/ViyLine`.

<br>

2. Install Rust
    - **Linux, MacOS**:
        - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
        - `rustup default stable`

    - **Windows**: [Go to this link](https://www.rust-lang.org/learn/get-started)

<br>

3. **Native** Compile and run ViyLine Rust
    - `cargo run --release`

<br>

4. **Web/WASM** Compile and run ViyLine on a browser
    - `cargo install trunk`
    - `trunk serve --release`
    - Web version does NOT have Bluetooth working.

<br>

5. *(optional)* Building the Assembly
    - You'll need to download MPLAB and compile the `.asm` code with `MPASMWIN.exe` to generate the `.hex` sent to the microcontroller.
    - Copy `mpasm` folder (that has `MPASMWIN.exe`, `p*.inc`) from installation to `./Assembly/thirdparty/mpasm`.
    - Either run `Compile.sh` in Linux/MacOS or on Windows open a PowerShell and run `MPASMWIN.exe /q /p16F877A "ViyLineCAP.asm"`

<br>



# ● Building the Circuit
The circuit


## List of Components

- 1x Microchip© PIC16F877A Microcontroller
- 1x HC-06 Bluetooth module
- 1x ManyFaradsAsPossible™ Capacitor *that doesn't kill you*
- 1x UART USB for writing the hexadecimal on the PIC
- A computer with bluetooth and any recent OS
