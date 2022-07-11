<div align="center">
    <a href="https://github.com/VernandoGames/road-render/actions"><img src="https://github.com/VernandoGames/road-render/workflows/CI/badge.svg" alt="Actions status" /></a>
</div>

**RoadRender** is a tool designed to render roblox place files in seconds.

The name **RoadRender** is a spinoff of the 'RoRender' project, indicating that it is a more advanced approach to the same general concept, while also working well as this tool was originally meant to render *just* roads.

## How does it work?

simply call the build-map command with a place file you would like to render and any necessary configurations, as highlighted below.

``road_render build-map --placefile ./path/to/place/file.rbxl --config ./path/to/config.json --center_x 0 --center_z 0 --height 0 --width 0 --scale 1``

### Argument highlights:
* --placefile - the .rbxl (or .rbxlx soon) that you would like to render
* --config - the config.json file you would like to use for the render
* --center_x - the x position for the exact center of the world render (soon to not be needed)
* --center_z - the z position for the exact center of the world render (soon to not be needed)
* --height - the height of the image to output
* --width - the width of the image to output
* --scale - world scale, useful for getting large areas in a single image.

## Example config
With this config, it will render all descendants of the folder ``Workspace.Map.Roads`` with the name ``Base`` and color it to ``RGBA(255, 255, 255, 255)`` following the RGBA color standard.
```json
{
	"draw_everything": false,
	"world_files": [
		{
			"dir": ["Workspace", "Map", "Roads"],
			"part_name": "Base",
			"color": [255, 255, 255, 255]
		}
	]
}
```


## Contributing
We don't yet have a fancy contribution guide setup, but you are more than welcome to try helping on the project!

Pull requests are welcome!

### Good starting points for contributors:
* automatically determine world size for rendering
* fit all contents of render within height and width of output image automatically
* Allow for rendering of terrain (not recommended for beginners)