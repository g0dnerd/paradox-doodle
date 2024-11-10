# Needs montage from ImageMagick in PATH
# Needs compressonatorcli.exe from https://github.com/GPUOpen-Tools/compressonator in PATH
# Needs PVRTexToolCLI.exe from https://developer.imaginationtech.com/pvrtextool/ in PATH

# Generate a skybox image from 6 jpeg in the folder in first argument.
# The images must be named right.jpg, left.jpg, top.jpg, bottom.jpg, back.jpg, front.jpg
#
# Must be called from the root of the project.
#
# bash examples/src/skybox/images/generation.bash ./path/to/images/folder

SCRIPT_DIRECTORY=src/assets/images
CHUNK_SIZE="4096X4096"

set -e

# ensure the script is called from the root of the project
if [ ! -f "$SCRIPT_DIRECTORY/generation.bash" ]; then
  echo "The script must be called from the root of the project!"
  exit 1
fi

# ensure an argument is passed
if [ $# -eq 0 ]; then
  echo "No arguments supplied!"
  echo
  echo "Usage: bash examples/src/skybox/images/generation.bash ./path/to/images/folder"
  exit 1
fi

TEMP=src/assets/images/tmp

mkdir -p $TEMP
# resize images to chunk size
mogrify -path $TEMP -resize $CHUNK_SIZE -format png $1/*.jpg

# create an uncompressed ktx2 cubemap file
echo "Combining PNGs to single texture"
PVRTexToolCLI -i $TEMP/right.png,$TEMP/left.png,$TEMP/top_right.png,$TEMP/bottom.png,$TEMP/front.png,$TEMP/back.png -ics SRGB -cube -m -f r8g8b8a8,UBN,SRGB -o $SCRIPT_DIRECTORY/rgba8.ktx2

# compressonator has support for etc2, but the result looks terrible.
# echo "Transcoding RGBa8 file to ETC2"
# PVRTexToolCLI -i $SCRIPT_DIRECTORY/rgba8.ktx2 -ics srgb -m -f ETC2_RGB_A1,UBN,SRGB -q etcslow -o $SCRIPT_DIRECTORY/etc2.ktx2
#
# echo "Transcoding RGBa8 file to ASTC"
# PVRTexToolCLI -i $SCRIPT_DIRECTORY/rgba8.ktx2 -ics srgb -m -f ASTC_4X4,UBN,SRGB -q astcexhaustive -o $SCRIPT_DIRECTORY/astc.ktx2
rm -rf $TEMP
