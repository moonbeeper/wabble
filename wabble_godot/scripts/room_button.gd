@tool
extends Button
class_name RoomButton

@export var title: RichTextLabel
@export var room_pop: Label
@export var room_icon: TextureRect
@export var room_pop_container: CenterContainer

var should_update: bool = true
enum TYPE {
	PUBLIC, PRIVATE, SETTINGS
}

@export var current_pop: int = 0:
	set(value):
		current_pop = value
		should_update = true
	get:
		return current_pop
@export var max_pop: int = 32:
	set(value):
		max_pop = value
		should_update = true
	get:
		return max_pop
@export var current_title: String = "Public Room #":
	set(value):
		current_title = value
		should_update = true
	get: 
		return current_title
@export var current_type: TYPE = TYPE.PUBLIC:
	set(value):
		current_type = value
		should_update = true
	get:
		return current_type
@export var should_hide_pop: bool = false:
	set(value):
		should_hide_pop = value
		should_update = true
	get:
		return should_hide_pop
		
var room_id: String = ""

func _ready() -> void:
	update_stuff()

func _process(_delta: float) -> void:
	if !should_update: return
	update_stuff()
		
	should_update = false

func update_stuff() -> void:
	print("updating")
	room_pop.text = "%s/%s" % [current_pop, max_pop]
	title.text = current_title
	var room_tex = AtlasTexture.new()
	room_tex.atlas = room_icon.texture.atlas
	match current_type:
		TYPE.PRIVATE:
			room_tex.region = Rect2(64.0, 0.0, 64.0, 64.0)
		TYPE.SETTINGS:
			room_tex.region = Rect2(128.0, 0.0, 64.0, 64.0)
		TYPE.PUBLIC:
			room_tex.region = Rect2(0.0, 0.0, 64.0, 64.0)
	room_icon.texture = room_tex
			
	if should_hide_pop:
		room_pop_container.visible = false
	else:
		room_pop_container.visible = true
