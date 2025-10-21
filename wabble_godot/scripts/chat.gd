extends Control

@export var note_card: NoteCard
@export var top_header: PanelHeader
@export var message_container: VBoxContainer
@export var message_scroll: SmoothScrollContainer

@export var send_button: Button
@export var get_button: Button
@export var erase_button: Button

var note_card_prefab = preload("res://nodes/note.tscn")

var pop_tween1: Tween
var pop_tween2: Tween
var pop_tween3: Tween

var message: String = ""
var recieved_message: String = ""
var recieved_drawing: PackedByteArray = PackedByteArray()

func _ready() -> void:
	top_header.original_position = top_header.position
	top_header.position = Vector2(0, top_header.position.y - top_header.size.y - 20)
	
	GameManager.recieved_message.connect(_on_message_recieved)
	note_card.set_username(GameManager.current_username)
	note_card.set_accent(GameManager.current_color)
	note_card.clear()
	
	top_header.titleText = GameManager.current_room_title
	top_header.tween_show()

	
func _on_send_pressed() -> void:
	do_pop_tween(pop_tween1, send_button)
	
	if message.is_empty() && note_card.is_drawing_empty():
		return
	
	var rle_drawing = note_card.get_drawing_rle()
	GameManager.send_message(message, rle_drawing)
	note_card.clear()
	message = ""
	
func _on_get_pressed() -> void:
	do_pop_tween(pop_tween2, get_button)
	
	message = recieved_message
	note_card.set_message(message)
	note_card.from_rle(recieved_drawing)
	
func _on_erase_pressed()-> void:
	do_pop_tween(pop_tween3, erase_button)
	
	note_card.clear()
	message = ""
	
func _input(event: InputEvent) -> void:
	if event is InputEventKey && event.is_pressed():
		var key_event = event as InputEventKey
		if key_event.keycode == KEY_BACKSPACE && message.length() > 0:
			message = message.substr(0, message.length() - 1)
			note_card.set_message(message)
		elif key_event.unicode:
			if message.length() > 165: return
			message += char(key_event.unicode)
			note_card.set_message(message)

func _on_message_recieved(mmessage: String, drawing: PackedByteArray, persona_name: String, persona_color: Color) -> void:
	print("recieved message")
	recieved_message = mmessage
	recieved_drawing = drawing
	
	var note = note_card_prefab.instantiate() as NoteCard
	note.set_accent(persona_color)
	message_container.add_child(note)
	note.set_username(persona_name)
	note.set_message(mmessage)
	note.from_rle(drawing)
	
	var tween = get_tree().create_tween()
	tween.tween_property(message_scroll, "scroll_vertical", message_scroll.get_v_scroll_bar().max_value, .5).from_current()
	await tween.finished

func do_pop_tween(tween: Tween, who: Node):
	if tween && tween.is_running():
		tween.kill()
	tween = get_tree().create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tween.tween_property(who, "modulate", Color(1.3, 1.3, 1.3, 1.0), 0.1).from_current()
	tween.chain().tween_property(who, "modulate", Color(1, 1, 1, 1), 0.2).from(Color(1.5,1.5,1.5,1))
