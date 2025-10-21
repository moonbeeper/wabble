extends Control

@export var top_header: PanelHeader
@export var bottom_header: PanelHeader
@export var line_edit: LineEdit

var new_username: String

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
	
	GameManager.fetched_persona.connect(_on_fetched_persona)
	GameManager.socket_ready.connect(_on_socket_ready)


func _on_confirm_pressed() -> void:
	GameManager.new_current_username = new_username
	GameManager.update_persona.emit()
	GameManager.send_opcode(6)
	tween_and_leave()
	

func _on_cancel_pressed() -> void:
	tween_and_leave()

func tween_and_leave() -> void:
	bottom_header.tween_hide()
	top_header.tween_hide()

	SceneManager.swap_scene("res://scenes/main_menu.tscn", self)
func _on_line_edit_text_changed(new_text: String) -> void:
	new_username = new_text

func _on_fetched_persona() -> void:
	line_edit.text = GameManager.current_username
	
func _on_socket_ready() -> void:
	GameManager.send_opcode(6)
