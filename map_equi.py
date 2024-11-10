from __future__ import print_function
import sys
import os.path
from PIL import Image
import zipfile

Image.MAX_IMAGE_PIXELS = 200500000

if len(sys.argv) < 2:
    print("Usage: cubemap-cut.py <filename.jpg|png>")
    sys.exit(-1)

infile = sys.argv[1]
filename, original_extension = os.path.splitext(infile)
file_extension = ".jpg"

name_map = [
    ["", "top", "", ""],
    ["left", "front", "right", "back"],
    ["", "bottom", "", ""],
]

try:
    im = Image.open(infile)
    print(infile, im.format, "%dx%d" % im.size, im.mode)

    width, height = im.size

    cube_size = width / 4

    filelist = []
    for row in range(3):
        for col in range(4):
            if name_map[row][col] != "":
                sx = cube_size * col
                sy = cube_size * row

                print(f"Indexing face {name_map[row][col]} at {{ x {sx}, y {sy}}}")

                filename = name_map[row][col] + file_extension
                filelist.append(filename)
                print(
                    f"Cropping {filename}: {(sx, sy, sx + cube_size, sy + cube_size)}"
                )
                face = im.crop((sx, sy, sx + cube_size, sy + cube_size))

                # Rotate top and bottom faces
                if name_map[row][col] in ["top", "bottom"]:
                    face = face.rotate(90 if name_map[row][col] == "top" else -90)

                face.save(filename)
                if name_map[row][col] == "bottom":
                    print("Bottom face stats:", face.getextrema())

    zfname = filename + ".zip"
    print("Creating zipfile: " + zfname)
    zf = zipfile.ZipFile(zfname, mode="w")
    try:
        for filename in filelist:
            zf.write(filename)
        print("done")
    finally:
        zf.close()

except IOError:
    pass
