👆【☰】Table of Contents

<div align="center">
  <img src="https://github.com/BrokenSource/ViyLine/raw/Master/Website/icon.png" onerror="this.src='../Website/icon.png'" width="256"/>

    ViyLine

  Solar panel IV curve tracker using a PIC16F877A Microcontroller, HC-06 Bluetooth module and a front end GUI in Rust.

</div>

<br>

<h5><b>NOTE:</b> This is a VERY DIY project, you will be building the physical circuit itself, we provide you the schematic and list of components.<h5>

<br>



# ● The Project: Briefly explained
- Every Solar Panel has a characteristic current-voltage curve it can output at either value being limited. The two magical points are the **open circuit voltage** and **short circuit voltage**.

- The curve changes with variations in luminosity and temperature, and can be used as a diagnosis of efficiency or any broken panel indication.

- The curve also gives the rated power output and the point of maximum efficiency, called the Maximum Power Point (MPP).

Read our [Paper](https://github.com/BrokenSource/ViyLine/raw/Master/Paper/Paper.pdf) for more details!



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



# ● License
All credits to Microchip© on their respective names of `PIC`, `MPASM`, included libraries.

ViyLine code falls under the `MIT` License. The logos, schematics are under CC-BY-4.0.



# ● Citing

A BibTeX entry for LaTeX users is:
```tex
@electronic{viyline,
    title = {ViyLine: A Solar Panel IV Curve Tracker},
    author = {Tremeschin},
    year = 2022,
    month = Dec,
    keywords = {Rust, PIC, Assembly, Solar Panel, IV Curve},
    abstract = {},
    url = {https://github.com/BrokenSource/ViyLine},
}
```