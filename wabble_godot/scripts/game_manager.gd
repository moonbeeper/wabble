extends Node

signal tick_update

var socket = WebSocketPeer.new()
var websocket_uri = "ws://127.0.0.1:8080/socket"
var rooms: Array = []
var server_population: int = 1
var update_tick: Timer
var is_socket_ok: bool = false


func _ready():
	socket.connect_to_url(websocket_uri)
	print("Connecting to WebSocket...")
	update_tick = Timer.new()
	update_tick.wait_time = 5.0
	update_tick.timeout.connect(_on_update_tick)
	add_child(update_tick)
	update_tick.start()

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
				tick_update.emit()
			7:
				print("recieved server pop")
				var recieved_data = data.get("data", {})
				server_population = recieved_data.get("pop", 0)
			8:
				print("recieved public room status")
				var recieved_data = data.get("data", {})
				rooms = recieved_data.get("public_rooms", [])
			_:
				print("unknown opcode recieved: ", opcode)
	else:
		push_error("somehow we failed to parse the recieved json: ", packet_text)

func _on_update_tick() -> void:
	if !is_socket_ok: return
	print("sending server pop reequest")
	var pop_message = {
		"op": 7
	}
	socket.send_text(JSON.stringify(pop_message))
	print("sending public room status reequest")
	var room_status_message = {
		"op": 8
	}
	socket.send_text(JSON.stringify(room_status_message))
	tick_update.emit()
