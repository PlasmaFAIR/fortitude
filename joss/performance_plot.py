import pathlib

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.ticker import StrMethodFormatter

data = {
    "Fortitude": 0.241,
    "Flint": 12.915,
    "Stylist": 14.830,
    "Camfort": 25.135,
}
data = dict(sorted(data.items(), key=lambda item: item[1]))

y_pos = np.arange(len(data))

fig, ax = plt.subplots(figsize=(8, 2))
ax.grid(axis="x")
bar = ax.barh(y_pos, data.values(), align="center", color="#744e97")
ax.bar_label(bar, fmt=" {:.2g}s")
ax.invert_yaxis()
ax.set_yticks(y_pos, labels=data.keys())
ax.xaxis.set_major_formatter(StrMethodFormatter("{x:.0f}s"))
ax.spines[:].set_visible(False)

fig.tight_layout()
fig.savefig("performance_plot.pdf")
