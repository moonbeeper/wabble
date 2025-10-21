extends Control

@export var server_pop: PanelHeader
@export var top_header: PanelHeader
@export var room_container: VBoxContainer

var room_button = preload("res://nodes/room_button.tscn")
var tracked_pub_rooms: Array[Node] = []
var already_shown_rooms: bool = false

func _ready() -> void:
	GameManager.tick_update.connect(_on_global_tick_update)
	server_pop.titleText = "Server Population: [wave]%s[/wave]" % GameManager.server_population
	
func _on_global_tick_update():
	server_pop.titleText = "Server Population: [wave]%s[/wave]" % GameManager.server_population
	
	if !tracked_pub_rooms.is_empty():
		for node in tracked_pub_rooms:
			node.queue_free()
		tracked_pub_rooms.clear()
	
	for room in GameManager.rooms:
		var button = room_button.instantiate() as RoomButton
		button.current_pop = int(room['active_connections'])
		button.max_pop = int(room['max_connections'])
		button.current_title = room['name']
		button.room_id = room['id']
		tracked_pub_rooms.append(button)
		room_container.add_child(button)
		if !already_shown_rooms:
			var tween = get_tree().create_tween().set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_CUBIC)
			tween.tween_property(button, "modulate", Color(1,1,1,1), .3).from(Color(1,1,1,0))
			await tween.finished
	already_shown_rooms = true


func _on_settings_pressed() -> void:
	server_pop.tween_hide()
	top_header.tween_hide()
	SceneManager.swap_scene("res://scenes/settings.tscn", self)
