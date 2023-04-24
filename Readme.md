👆【☰】Table of Contents

<div align="center">
  <img src="https://github.com/BrokenSource/ViyLine/raw/Master/ViyLine/icon.png" onerror="this.src='../ViyLine/icon.png'" width="256"/>

    ViyLine - Vee Aye Line

  Solar panel IV curve tracker using Arduino and a front end GUI in Rust.

</div>

<br>

<sub><b>NOTE:</b> This is a VERY DIY project, you will be building the physical circuit itself, we provide you the schematic and list of components.</sub>

<br>

<img src="https://user-images.githubusercontent.com/29046864/206887190-394abf94-4711-4895-99ae-ba2229928477.png"/>



# ● The Project: Briefly explained
- Every Solar Panel has a characteristic current-voltage curve it can output at either value being limited. The two magical points are the **open circuit voltage** and **short circuit voltage**.

- The curve changes with variations in luminosity and temperature, and can be used as a diagnosis of efficiency or any broken panel indication.

- The curve also gives the rated power output and the point of maximum efficiency, called the Maximum Power Point (MPP).

Collecting these points requires a variable load that ranges from short circuit to open circuit and the ability to measure pairs of voltage and current points throughout the process, sweeping all possible loads.

A microcontroller is used to measure the points and send signals to MOSFETs to either charge or discharge a capacitor, acting as a variable load when charging.

The points are sent back to the Rust code using either Serial or Bluetooth communication, and a non linear regression is applied to generate the best prediction of the real IV curve being measured.


# ● Running from Source Code
See the [BrokenSource](https://github.com/Tremeschin/BrokenSource) for setting up the development environment.

- Compile and upload the `ViyLine.ino` Arduino code to an Arduino UNO board
- Run: `broken viyline`


# ● Building the Circuit
The circuit schematic will be provided at some point in the close future.


# ● License

MIT License. Schematics under CC-BY-4.0. See the [BrokenSource](https://github.com/Tremeschin/BrokenSource) for more details.


# ● Citing

BibTeX entry for LaTeX:
```bibtex
@electronic{viyline,
  title    = {ViyLine: Traçador de Curva IV do Painel Solar},
  keywords = {Rust, Arduino, Solar Panel, IV Curve},
  url      = {https://github.com/BrokenSource/ViyLine},
  author   = {Tremeschin; et al.},
  year     = {2022},
  abstract = {
    Pronunciado Vee-Aye-Line, este software escrito em
    Assembly e Rust, disponibilizado em Código Aberto, utilizando
    Arduino Uno coleta pontos da Curva IV característica de um Painel
    Solar com tensões até 50V e corrente até 5A. Seu circuito de medição
    é composto por capacitores, que atuam como uma carga variável para
    a coleta de pontos da curva. Também possui uma interface gráfica e
    funciona com cabo USB Serial UART ou pelo módulo Bluetooth
    HC-06 para o envio de sinais e recebimento de dados. Possui
    uma modelagem matemática por métodos computacionais para melhor
    precisão dos dados exportados pela ferramenta, realizando uma
    regressão não linear para ajuste de curva dos pontos medidos
    em relação à curva teórica de operação de um painel solar, a
    fim de mitigar imprecisões de medição utilizando-se da estatística.
  },
}
```