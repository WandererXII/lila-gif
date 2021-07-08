import xml.etree.ElementTree as ET
import chess.svg

import pieces as TP

COLORS = [
    "#fabb54", # light square
    "#d3bf31", # highlighted light square
    "#262421", # dark background
    "#bababa", # text color
    "#bf811d", # title color
    "#b72fc6", # bot color
    "#706f6e", # 50% text color on dark background
    "#6b6b6b", # pocket background color
    "#ffffff", # white text
]

PIECES_NAMES = {
    0:  "king",
    1:  "rook",
    2:  "bishop",
    3:  "gold",
    4:  "silver",
    5:  "knight",
    6:  "lance",
    7:  "pawn",
    8:  "dragon",
    9:  "horse",
    10: "prosilver",
    11: "proknight",
    12: "prolance",
    13: "tokin",
    14: "tama"
}

HAND_PIECES = ["pawn", "lance", "knight", "silver", "gold", "bishop", "rook"]

SQUARE_SIZE = 100
COLOR_WIDTH = SQUARE_SIZE


def make_sprite(f):
    svg = ET.Element("svg", {
        "xmlns": "http://www.w3.org/2000/svg",
        "version": "1.1",
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        "viewBox": f"0 0 {SQUARE_SIZE * 12} {SQUARE_SIZE * 9}",
    })

    defs = ET.SubElement(svg, "defs")
    defs.append(ET.fromstring(TP.PIECE_BACKGROUND))
    for g in TP.PIECES.values():
        defs.append(ET.fromstring(g))

    defs.append(ET.fromstring(chess.svg.CHECK_GRADIENT))

    for x, color in enumerate(COLORS[2:]):
        ET.SubElement(svg, "rect", {
            "x": str(COLOR_WIDTH * x),
            "y": "0",
            "width": str(COLOR_WIDTH),
            "height": str(SQUARE_SIZE),
            "stroke": "none",
            "fill": color,
        })
        
    ET.SubElement(svg, "rect", {
        "x": str(SQUARE_SIZE * 8),
        "y": str(SQUARE_SIZE * 2),
        "width": str(SQUARE_SIZE * 4),
        "height": str(7 * SQUARE_SIZE),
        "stroke": "#6b6b6b",
        "fill": "#6b6b6b",
    })

    for x in range(4):
        for y, piece in enumerate(HAND_PIECES):
            color = "white" if x % 2 else "black"
            opacity = "1.0" if x > 1 else "0.1"
            ET.SubElement(svg, "use", {
                    "xlink:href": f"#{color}-{piece}",
                    "transform": f"translate({SQUARE_SIZE * (8 + x)}, {SQUARE_SIZE * (y + 2)})",
                    "opacity": opacity,
                    "x": "0",
                    "y": "0"
                })
            if x > 1:
                ET.SubElement(svg, "rect", {
                    "transform": f"translate({SQUARE_SIZE * (8 + x) + (SQUARE_SIZE * 2 / 3)}, {SQUARE_SIZE * (y + 2) + (SQUARE_SIZE * 2 / 3)})",
                    "width": str(SQUARE_SIZE / 3),
                    "height": str(SQUARE_SIZE / 3),
                    "stroke": "none",
                    "rx": "10",
                    "fill": "#d64f00",
                })

    ET.SubElement(svg, "circle", {
        "cx": str(SQUARE_SIZE * 10 + SQUARE_SIZE / 2),
        "cy": str(SQUARE_SIZE + SQUARE_SIZE / 2),
        "r": str(SQUARE_SIZE / 10),
        "fill": "#000",
    })

    for x in range(12):
        for y in range(9):

            if( y != 0 or x > 7) and (x < 8 or y < 2) and not (x > 9 and y == 1):
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_SIZE * x),
                    "y": str(SQUARE_SIZE * y),
                    "width": str(SQUARE_SIZE),
                    "height": str(SQUARE_SIZE),
                    "stroke": "#000",
                    "stroke-width": "2",
                    "fill": COLORS[x % 2],
                })

            if y > 4 and (x == 6 or x == 7):
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_SIZE * x),
                    "y": str(SQUARE_SIZE * y),
                    "width": str(SQUARE_SIZE),
                    "height": str(SQUARE_SIZE),
                    "fill": "url(#check_gradient)",
                })
            if y > 0 and x < 8:
                my_sq = (y-1)//2 + x//2 * 4
                piece_type = my_sq % 15
                color = "white" if y % 2 else "black"
                ET.SubElement(svg, "use", {
                    "xlink:href": f"#{color}-{PIECES_NAMES[piece_type]}",
                    "transform": f"translate({SQUARE_SIZE * x}, {SQUARE_SIZE * y})",
                    "x": "0",
                    "y": "0"
                })
            elif y < 2 and (x == 8 or x == 9):
                color = "white" if (y+1) % 2 else "black"
                ET.SubElement(svg, "use", {
                    "xlink:href": f"#{color}-tama",
                    "transform": f"translate({SQUARE_SIZE * x}, {SQUARE_SIZE * y})",
                    "x": "0",
                    "y": "0"
                })

    f.write(ET.tostring(svg))

if __name__ == "__main__":
    make_sprite(open("sprite.svg", "wb"))
