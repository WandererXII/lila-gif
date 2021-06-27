import xml.etree.ElementTree as ET
import chess.svg

import pieces as TP

COLORS = [
    "#f0d9b5", # light square
    "#ced26b", # highlighted light square
    "#262421", # dark background
    "#bababa", # text color
    "#bf811d", # title color
    "#b72fc6", # bot color
    "#706f6e", # 50% text color on dark background
]

SQUARE_SIZE = 100
COLOR_WIDTH = SQUARE_SIZE * 2 // 3


def make_sprite(f):
    svg = ET.Element("svg", {
        "xmlns": "http://www.w3.org/2000/svg",
        "version": "1.1",
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        "viewBox": f"0 0 {SQUARE_SIZE * 9} {SQUARE_SIZE * 9}",
    })

    defs = ET.SubElement(svg, "defs")
    defs.append(ET.fromstring(TP.PIECE_BACKGROUND))
    for g in TP.PIECES.values():
        defs.append(ET.fromstring(g))
    #defs.append(ET.fromstring(TP.PIECES['K']))
    #defs.append(ET.fromstring(TP.PIECES['P']))

    defs.append(ET.fromstring(chess.svg.CHECK_GRADIENT))

    for x, color in enumerate(COLORS[2:]):
        ET.SubElement(svg, "rect", {
            "x": str(SQUARE_SIZE * 5 + COLOR_WIDTH * x),
            "y": "0",
            "width": str(COLOR_WIDTH),
            "height": str(SQUARE_SIZE),
            "stroke": "none",
            "fill": color,
        })

    PIECES_NAMES = {
        0: "gold",
        1: "pawn",
        2: "",
        3: "",
        4: "bishop",
        5: "lance",
        6: "prosilver",
        7: "tokin",
        8: "rook",
        9: "knight",
        10: "horse",
        11: "prolance",
        12: "king",
        13: "silver",
        14: "dragon",
        15: "proknight",
    }

    for x in range(9):
        for y in range(9):
            my_sq = y//2 * 4 + x//2
            piece_type = my_sq if my_sq < 16 else 12
            color = "white" if y % 2 else "black"

            if y != 0 or x < 5:
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_SIZE * x),
                    "y": str(SQUARE_SIZE * y),
                    "width": str(SQUARE_SIZE),
                    "height": str(SQUARE_SIZE),
                    "stroke": "#000",
                    "fill": COLORS[1] if y == 0 and x == 4 else COLORS[x % 2],
                })

            if y == 8 and x != 8:
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_SIZE * x),
                    "y": str(SQUARE_SIZE * y),
                    "width": str(SQUARE_SIZE),
                    "height": str(SQUARE_SIZE),
                    "fill": "url(#check_gradient)",
                })
            if x < 8:
                ET.SubElement(svg, "use", {
                    "xlink:href": f"#{color}-{PIECES_NAMES[piece_type]}",
                    "transform": f"translate({SQUARE_SIZE * x}, {SQUARE_SIZE * y})",
                    "x": "0",
                    "y": "0"
                })

    f.write(ET.tostring(svg))

if __name__ == "__main__":
    make_sprite(open("sprite_new.svg", "wb"))
