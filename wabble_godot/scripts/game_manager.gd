extends Node
class_name SGameManager

signal tick_update
signal color_change(id: COLOR)
signal update_persona
signal socket_ready
signal fetched_persona
signal swap_scene(res: String)
signal recieved_message(message: String, drawing: PackedByteArray, persona_name: String, persona_color: Color)

var socket = WebSocketPeer.new()
# the idea was to let the user change the uri but nope :)
var websocket_uri = "ws://127.0.0.1:8080/socket"
var rooms: Array = []
var server_population: int = 1
var update_tick: Timer
var is_socket_ok: bool = false

var current_color: Color = Color(1,1,1,1)
var current_username: String = "user123456"

var new_current_color: COLOR = COLOR.PURPLE
var new_current_username: String

var current_room_title: String = "Unknown room"

enum COLOR {
	RED, ORANGE, PURPLE, LIGHT_GREEN, GREEN, LIGHT_BLUE, BLUE, NOTBLUE
}

func _ready():
	socket.connect_to_url(websocket_uri)
	print("Connecting to WebSocket...")
	update_tick = Timer.new()
	update_tick.wait_time = 5.0
	update_tick.timeout.connect(_on_update_tick)
	add_child(update_tick)
	update_tick.start()
	
	color_change.connect(_on_color_change)
	update_persona.connect(_on_update_persona)

func _process(_delta: float) -> void:
	socket.poll()  
	var state = socket.get_ready_state()
	
	match state:
		WebSocketPeer.STATE_OPEN:
			is_socket_ok = true
			while socket.get_available_packet_count():
				var packet = socket.get_packet()
				if socket.was_string_packet():
					var packet_text = packet.get_string_from_utf8()
					handle_message(packet_text)
		WebSocketPeer.STATE_CLOSING:
			is_socket_ok = false		
			print("socket is closing")
		WebSocketPeer.STATE_CLOSED:
			is_socket_ok = false
			var code = socket.get_close_code()
			print("WebSocket closed with code: %d. Clean: %s" % [code, code != -1])
			socket.connect_to_url(websocket_uri) # reconnect i guess

func handle_message(packet_text: String) -> void:
	var json = JSON.new()
	var parse_result = json.parse(packet_text)
	
	if parse_result == OK:
		var data = json.data
		var opcode: int = data.get("op")
		
		match opcode:
			0:
				var recieved_data = data.get("data", {})
				print("recieved handshake")
				print(recieved_data)
				server_population = recieved_data.get("active_connections", 1)
				rooms = recieved_data.get("public_rooms", []) 
				socket_ready.emit()
				send_opcode(6)
				_on_update_tick()
			7:
				print("recieved server pop")
				var recieved_data = data.get("data", {})
				server_population = recieved_data.get("pop", 0)
			8:
				print("recieved public room status")
				var recieved_data = data.get("data", {})
				rooms = recieved_data.get("public_rooms", [])
			6:
				print("recieved whoami")
				var recieved_data = data.get("data", {})
				var persona = recieved_data.get("persona", {})
				var color = persona.get("color", 0xFFFFFFFF)
				print(color)
				current_color = Color.from_string(color, Color(1,1,1,1))
				current_username = persona.get("name", "user12345")
				print(current_color)
				print(current_username)
				fetched_persona.emit()
			4:
				print("recieved echo message")
				var recieved_data = data.get("data", {})
				print(data)
				var message = recieved_data.get("message", "")
				var persona = recieved_data.get("persona", {})
				var persona_name = persona.get("name", "unknown_usr")
				var raw_persona_color = persona.get("color", "FFFFFFFF")
				var persona_color = Color.from_string(raw_persona_color, Color(1,1,1,1))
				var raw_drawing = recieved_data.get("drawing", "")
				if SceneManager.in_progress_transition: await SceneManager.scene_ready
				if raw_drawing != null and raw_drawing is String and raw_drawing != "":
					var drawing = Marshalls.base64_to_raw(raw_drawing)
					recieved_message.emit(message, drawing, persona_name, persona_color)
				else:
					recieved_message.emit(message, PackedByteArray(), persona_name, persona_color)

			_:
				print("unknown opcode recieved: ", opcode)
	else:
		push_error("somehow we failed to parse the recieved json: ", packet_text)

func _on_update_tick() -> void:
	if !is_socket_ok: return
	print("sending server pop reequest")
	send_opcode(7)
	print("sending public room status reequest")
	send_opcode(8)
	tick_update.emit()

func send_opcode(code: int) -> void:
	var message = {
		"op": code
	}
	socket.send_text(JSON.stringify(message))

func _on_color_change(id: COLOR) -> void:
	new_current_color = id

func _on_update_persona() -> void:
	current_color = _from_color_enum(new_current_color)
	current_username = new_current_username
	
	var message = {
		"op": 1,
		"data": {
			"name": current_username,
			"color": current_color.to_html()
		}
	}
	socket.send_text(JSON.stringify(message))
	
	pass

func _from_color_enum(id: COLOR) -> Color:
	match id:
		COLOR.RED:
			return Color(1.0, 0.42, 0.42)
		COLOR.ORANGE:
			return Color(1.0, 0.686, 0.302)
		COLOR.PURPLE:
			return Color(0.984, 0.541, 0.984)
		COLOR.LIGHT_GREEN:
			return Color(0.667, 0.984, 0.0)
		COLOR.GREEN:
			return Color(0.0, 0.635, 0.22)
		COLOR.BLUE:
			return Color(0.0, 0.349, 0.953)
		COLOR.NOTBLUE:
			return Color(0.38, 0.51, 0.604)
		COLOR.LIGHT_BLUE:
			return Color(0.188, 0.729, 0.953)
		_:
			return Color(0.984, 0.541, 0.984)

func join_room(id: String, is_private: bool) -> void:
	var message = {
		"op": 2,
		"data": {
			"id": id
		}
	}
	socket.send_text(JSON.stringify(message))
	var room_info = null
	for room in rooms:
		if room.get("id") == id:
			room_info = room
			break
	print(room_info)
	if room_info:
		current_room_title = room_info.get("name", "Unknown room")
		print("Joined room: ", current_room_title)
	else:
		if is_private:
			current_room_title = "Private room :o"
		else:
			print("Room not found!")

func signal_swap_scene(res: String) -> void:
	swap_scene.emit(res)
	
func send_message(message: String, drawing: PackedByteArray) -> void:
	var base_drawing = Marshalls.raw_to_base64(drawing)
	var payload = {
		"op": 3,
		"data": {
			"message": message,
			"drawing": base_drawing
		}
	}
	socket.send_text(JSON.stringify(payload))
