from rich.color import Color
from rich.console import Console
from rich.style import Style
from rich.text import Text

console = Console()


# left gradient: #fe5296 --> rgb(254, 82, 150)
# right gradient: #f77063 --> rgb(247, 112, 99)
def main(text, leftRgb=(254, 82, 150), rightRgb=(247, 112, 99), fgRgb=(0, 0, 0)):
    # startHex = "fe5296"
    # endHex = "f77063"
    # start = tuple(int(startHex.lstrip("#")[i : i + 2], 16) for i in (0, 2, 4))
    # end = tuple(int(endHex.lstrip("#")[i : i + 2], 16) for i in (0, 2, 4))
    length = max(len(text) - 1, 1)

    styled = Text()
    for i, char in enumerate(text):
        gradientPosition = i / length
        r = round(leftRgb[0] + (rightRgb[0] - leftRgb[0]) * gradientPosition)
        g = round(leftRgb[1] + (rightRgb[1] - leftRgb[1]) * gradientPosition)
        b = round(leftRgb[2] + (rightRgb[2] - leftRgb[2]) * gradientPosition)

        style = Style(
            color=Color.from_rgb(fgRgb[0], fgRgb[1], fgRgb[2]),
            bgcolor=Color.from_rgb(r, g, b),
        )

        styled.append(char, style=style)

    console.print(styled)


if __name__ == "__main__":
    main(text="test text; you should probably replace this")
