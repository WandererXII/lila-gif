import xml.etree.ElementTree as ET
import chess.svg

import dist.pieces as TP

COLORS = [
    "#f4bf57", # square
    "#d0c333", # highlighted square
    "#262421", # dark background
    "#bababa", # text color
    "#bf811d", # title color
    "#b72fc6", # bot color
    "#706f6e", # 50% text color on dark background
    "#6a6b6b", # pocket background color
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

SCALE = 7
SQUARE_WIDTH = 11 * SCALE
SQUARE_HEIGHT = 12 * SCALE


def make_sprite(f):
    svg = ET.Element("svg", {
        "xmlns": "http://www.w3.org/2000/svg",
        "version": "1.1",
        "xmlns:xlink": "http://www.w3.org/1999/xlink",
        "viewBox": f"0 0 {SQUARE_WIDTH * 12} {SQUARE_HEIGHT * 9}",
    })

    defs = ET.SubElement(svg, "defs")
    defs.append(ET.fromstring(TP.PIECE_BACKGROUND))
    for g in TP.PIECES.values():
        defs.append(ET.fromstring(g))

    defs.append(ET.fromstring(chess.svg.CHECK_GRADIENT))

    for x, color in enumerate(COLORS[2:]):
        ET.SubElement(svg, "rect", {
            "x": str(SQUARE_WIDTH * x),
            "y": "0",
            "width": str(SQUARE_WIDTH),
            "height": str(SQUARE_HEIGHT),
            "shape-rendering": "crispEdges",
            "stroke": "none",
            "fill": color,
        })
        
    ET.SubElement(svg, "rect", {
        "x": str(SQUARE_WIDTH * 8),
        "y": str(SQUARE_HEIGHT * 2),
        "width": str(SQUARE_WIDTH * 4),
        "height": str(7 * SQUARE_HEIGHT),
        "stroke": "none",
        "fill": COLORS[7],
    })

    for x in range(4):
        for y, piece in enumerate(HAND_PIECES):
            color = "white" if x % 2 else "black"
            opacity = "1.0" if x > 1 else "0.1"
            wrap = ET.SubElement(svg, 'svg', {
                "x": str(SQUARE_WIDTH * (8 + x)),
                "y": str(SQUARE_HEIGHT * (y + 2)),
                "width": str(SQUARE_WIDTH),
                "height": str(SQUARE_HEIGHT),
                "viewBox": "5 5 90 90"
            })
            ET.SubElement(wrap, "use", {
                "opacity": opacity,
                "xlink:href": f"#{color}-{piece}",
            })
            if x > 1:
                ET.SubElement(svg, "rect", {
                    "transform": f"translate({SQUARE_WIDTH * (8 + x) + (SQUARE_WIDTH / 5 * 3)}, {SQUARE_HEIGHT * (y + 2) + (SQUARE_HEIGHT / 5 * 3)})",
                    "width": str(SQUARE_WIDTH / 5 * 2),
                    "height": str(SQUARE_WIDTH / 5 * 2), # we want a square
                    "stroke": "#fff",
                    "stroke-width": "2",
                    "rx": "8",
                    "fill": COLORS[2],
                })

    ET.SubElement(svg, "circle", {
        "cx": str(SQUARE_WIDTH * 10 + SQUARE_WIDTH / 2),
        "cy": str(SQUARE_HEIGHT + SQUARE_HEIGHT / 2),
        "r": str(5),
        "fill": "#000",
    })

    for x in range(12):
        for y in range(9):

            if( y != 0 or x > 7) and (x < 8 or y < 2) and not (x > 9 and y == 1):
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_WIDTH * x),
                    "y": str(SQUARE_HEIGHT * y),
                    "width": str(SQUARE_WIDTH),
                    "height": str(SQUARE_HEIGHT),
                    "stroke": "#000",
                    "stroke-width": "2",
                    "fill": COLORS[x % 2],
                })

            if y > 4 and (x == 6 or x == 7):
                ET.SubElement(svg, "rect", {
                    "x": str(SQUARE_WIDTH * x),
                    "y": str(SQUARE_HEIGHT * y),
                    "width": str(SQUARE_WIDTH),
                    "height": str(SQUARE_HEIGHT),
                    "fill": "url(#check_gradient)",
                })
            if y > 0 and x < 8:
                my_sq = (y-1)//2 + x//2 * 4
                piece_type = my_sq % 15
                color = "white" if y % 2 else "black"
                wrap = ET.SubElement(svg, 'svg', {
                    "x": str(SQUARE_WIDTH * x),
                    "y": str(SQUARE_HEIGHT * y),
                    "width": str(SQUARE_WIDTH),
                    "height": str(SQUARE_HEIGHT),
                    "viewBox": "5 5 90 90"
                })
                ET.SubElement(wrap, "use", {
                    "xlink:href": f"#{color}-{PIECES_NAMES[piece_type]}",
                })
            elif y < 2 and (x == 8 or x == 9):
                color = "white" if (y+1) % 2 else "black"
                wrap = ET.SubElement(svg, 'svg', {
                    "x": str(SQUARE_WIDTH * x),
                    "y": str(SQUARE_HEIGHT * y),
                    "width": str(SQUARE_WIDTH),
                    "height": str(SQUARE_HEIGHT),
                    "viewBox": "5 5 90 90"
                })
                ET.SubElement(wrap, "use", {
                    "xlink:href": f"#{color}-tama",
                })

    f.write(ET.tostring(svg))

if __name__ == "__main__":
    make_sprite(open("sprite.svg", "wb"))
