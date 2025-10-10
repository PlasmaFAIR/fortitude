import pathlib

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.ticker import StrMethodFormatter

TOP_LEVEL = pathlib.Path(__file__).parent.parent
DOCS_IMAGE_DIR = TOP_LEVEL / "docs" / "assets"

data = {
    "Fortitude": 0.095,
    "Flint": 7.387,
    "Stylist": 10.417,
    "Camfort": 6.609,
    "iCode": 21.582,
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

fig.savefig(DOCS_IMAGE_DIR / "performance_plot_light.svg", transparent=True)

ax.tick_params(labelcolor="lightgrey")
ax.bar_label(bar, fmt=" {:.2g}s", color="lightgrey")
fig.savefig(DOCS_IMAGE_DIR / "performance_plot_dark.svg", transparent=True)
