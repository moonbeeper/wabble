extends Control

@export var top_header: PanelHeader
@export var bottom_header: PanelHeader
@export var room_id: LineEdit

var room_id_str: String

func _ready() -> void:
	_do_header_tween.call_deferred()
	
func _do_header_tween() -> void:
	top_header.original_position = top_header.position
	bottom_header.original_position = bottom_header.position
	top_header.position = Vector2(0, top_header.position.y - top_header.size.y - 20)
	bottom_header.position = Vector2(0, bottom_header.position.y + bottom_header.size.y + 20)
	print(bottom_header.original_position)
	
	top_header.tween_show()
	bottom_header.tween_show()

func _on_line_edit_text_changed(new_text: String) -> void:
	room_id_str = new_text

func _on_cancel_join_pressed() -> void:
	SceneManager.swap_scene("res://scenes/main_menu.tscn", self)

func _on_confirm_join_pressed() -> void:
	var regex = RegEx.new()
	regex.compile("^[a-zA-Z0-9]{3}(?:-[a-zA-Z0-9]{3})*$")
	if regex.search(room_id_str):
		GameManager.join_room(room_id_str, true, false)
		SceneManager.swap_scene("res://scenes/chat.tscn", self)
	else:
		room_id_str = ""
		room_id.text = ""
	

func _on_cancel_create_pressed() -> void:
	SceneManager.swap_scene("res://scenes/main_menu.tscn", self)

func _on_confirm_create_pressed() -> void:
	GameManager.join_room("", true, true)
	SceneManager.swap_scene("res://scenes/chat.tscn", self)
