"""julia_with_rs.py"""

import time

from julia_set_rs import calculate_z_serial
from line_profiler import profile

# area of complex space to investigate
x1, x2, y1, y2 = -1.8, 1.8, -1.8, 1.8
c_real, c_imag = -0.62772, -0.42193


@profile
def calc_with_rs(desired_width: float, max_iterations: int) -> None:
    """Create a list of complex coordinates (zs) and complex parameters (cs), build Julia set"""

    x_step: float = (x2 - x1) / desired_width
    y_step: float = (y1 - y2) / desired_width
    x: list[float] = []
    y: list[float] = []

    ycoord: float = y2

    while ycoord > y1:
        y.append(ycoord)
        ycoord += y_step

    xcoord: float = x1

    while xcoord < x2:
        x.append(xcoord)
        xcoord += x_step

    # build a list of coordinates and the initial condition for each cell.
    zs: list[complex] = []
    cs: list[complex] = []

    for ycoord in y:
        for xcoord in x:
            zs.append(complex(xcoord, ycoord))
            cs.append(complex(c_real, c_imag))

    print(f"Length of x: {len(x)}")
    print(f"Total elements: {len(zs)}")

    start_time: float = time.perf_counter()

    output: list[int] = calculate_z_serial(max_iterations, zs, cs)

    end_time: float = time.perf_counter()

    print(f"{calculate_z_serial.__name__} took {end_time - start_time} seconds.")

    assert sum(output) == 33_219_980


if __name__ == "__main__":
    calc_with_rs(desired_width=1_000, max_iterations=300)
