@tool
extends Button
class_name ColorButton

@export var color_rect: ColorRect
@export var selected_bg: ColorRect
@export var color_id: GameManager.COLOR = GameManager.COLOR.PURPLE

var should_update: bool = true
@export var is_selected: bool = false:
	set(value):
		is_selected = value
		should_update = true
	get:
		return is_selected
	

@export var current_color: Color = Color(1.0, 0.34, 0.901, 1.0)

func _ready() -> void:
	color_rect.color = current_color
	selected_bg.visible = is_selected
	GameManager.color_change.connect(_on_color_change)

func _process(_delta: float) -> void:
	if !should_update: return
	
	selected_bg.visible = is_selected
	should_update = false

func _on_pressed() -> void:
	print("color button pressed")
	GameManager.color_change.emit(color_id)
	
func _on_color_change(id: GameManager.COLOR) -> void:
	if id != color_id:
		is_selected = false
		return
	is_selected = true
