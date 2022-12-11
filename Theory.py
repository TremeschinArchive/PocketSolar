#!/usr/bin/python
# | (c) 2022 Tremeschin, MIT License | ViyLine Project | #

# -------------------------------------------------------|

# Script for showcasing the theory on capacitor acting as
# a variable load for measuring an IV curve from a solar
# panel, using differential equations

# Need to install:
# :: pip install plotly numpy

# -------------------------------------------------------|

# Imports
from plotly.subplots import make_subplots
import plotly.graph_objects as go
from numpy import *

class Simulation:
    VoltageSource = 0
    SolarPanel    = 1

# -------------------------------------------------------|

# Parameters from the IV curve
# :: I(V) = IL - IOexp(qV/nkT)
# :: y = A - B exp(Cx)
Afv = 10
Bfv = 0.0016
Cfv = 0.165

# Capacitance of the capacitor
C = 7000 * 10**-6

# Resistance of the resistor, only used in VoltageSource
R = 1

# Differential time to simulate in Euler's method
dt = 0.00001

# Initial charge of the capacitor
Q = 0

# # What to connect the capacitor with
simulation = Simulation.SolarPanel

# The voltage source voltage
voltageSourceVoltage = 0.5

# -------------------------------------------------------|

# Given a current or voltage, get the other
# point on our target IV curve synthetically
def I(V): return Afv - Bfv*exp(Cfv*V)
def V(I): return log((Afv - I)/Bfv)/Cfv

# Measurements
Vcapacitor = []
Icapacitor = []
Vsource = []
Qcap = []
timeAxis = []

VIx = []
VIy = []

# -------------------------------------------------------|

# For each t spaced by dt up untill 0.6 seconds
for iteration, t in enumerate(arange(0, 0.07, dt)):

    # The energy stored on a capacitor is Q = VC
    # so the voltage across the capacitor is Q/C
    Vcap = (Q/C)

    # Voltage source + R + C
    if simulation == Simulation.VoltageSource:
        Vf = voltageSourceVoltage

        # Current follows Ohm's laws
        i = (Vf - Vcap)/R

    # Solar panel + C without R
    elif simulation == Simulation.SolarPanel:

        # Current is the current at capacitor's current voltage
        i = I(Vcap)

        # And voltage is IV curve point given the current
        Vf = V(i)

    else:
        raise RuntimeError("Invalid Simulation")

    # Since i = dQ/dt, dQ = i * dt
    # Add dQ to the capacitor charge
    Q += dt*i

    # Append data every 10 points
    if iteration % 10 == 0:
        timeAxis.append(t)
        Vcapacitor.append(Vcap)
        Icapacitor.append(i)
        Vsource.append(Vf)
        Qcap.append(Q)

        # IV curve measured points
        VIx.append(Vf)
        VIy.append(i)

# -------------------------------------------------------|
# Constructing plotly figure

# Make subplots
fig = make_subplots(rows=2, cols=1)
fig.layout.font.size = 24

# Syntactic sugar
def plot(name, x, y, row=1):
    fig.add_trace(go.Scatter(
        x=x, y=y,
        name=name,
        mode="markers",
        showlegend=True,
        line=dict(width=6),
    ), row=row, col=1)

# -------------------------------------------------------|
# Figure 1: Time domain stuff

# Curves to plot
toPlot = [
    ("V Cap", Vcapacitor),
    ("I Cap", Icapacitor),
    ("V Source", Vsource),
    ("Q Cap", Qcap)
]

for name, data in toPlot:
    plot(name, timeAxis, data)

# -------------------------------------------------------|
# Figure 1: IV curve

plot("IV Curve", VIx, VIy, 2)

fig.show()
