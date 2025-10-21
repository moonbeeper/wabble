extends Control
class_name NoteCard

# mr gpt helped a lot with the drawing into a image tex. pretty nifty

@export var line1: Label
@export var line2: Label
@export var line3: Label
@export var line4: Label
@export var line5: Label
@export var username: Label
@export var username_bg: NinePatchRect
@export var drawing_layer: TextureRect 
@export var card_mask: TextureRect


@export var draw_thickness: float = 2
@export var erase_thickness: float = 8.0 

enum DrawMode {
	DRAW,
	ERASE
}

@export var drawing_enabled: bool = false
var is_drawing: bool = false
var last_draw_pos: Vector2
var drawing_image: Image
@export var current_mode: DrawMode = DrawMode.DRAW

func _ready():
	_setup_drawing()
	#set_note("moonbeeper", "avalis are pretty cool ngl wawa")
	#enable_drawing()
	#set_draw_mode(NoteCard.DrawMode.DRAW) 

func _setup_drawing():	
	drawing_image = Image.create(int(drawing_layer.size.x), int(drawing_layer.size.y), false, Image.FORMAT_RGBA8)
	drawing_image.fill(Color(0, 0, 0, 0))
	
	drawing_layer.texture = ImageTexture.create_from_image(drawing_image)

func enable_drawing():
	drawing_enabled = !drawing_enabled

func set_draw_mode(mode: DrawMode):
	current_mode = mode
	
func _gui_input(event: InputEvent):
	if !drawing_enabled: return
	
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			is_drawing = event.pressed
			if is_drawing:
				last_draw_pos = event.position
	elif event is InputEventMouseMotion and is_drawing:
		_draw_line_on_image(last_draw_pos, event.position)
		last_draw_pos = event.position
		drawing_layer.texture = ImageTexture.create_from_image(drawing_image)
		
# what 
func _draw_line_on_image(from: Vector2, to: Vector2):
	var x0 = int(from.x)
	var y0 = int(from.y)
	var x1 = int(to.x)
	var y1 = int(to.y)
	
	var dx = abs(x1 - x0)
	var dy = abs(y1 - y0)
	var sx = 1 if x0 < x1 else -1
	var sy = 1 if y0 < y1 else -1
	var err = dx - dy
	
	while true:
		_draw_pixel(x0, y0)
		
		if x0 == x1 and y0 == y1:
			break
		
		var e2 = 2 * err
		if e2 > -dy:
			err -= dy
			x0 += sx
		if e2 < dx:
			err += dx
			y0 += sy
			
func _draw_pixel(x: int, y: int):
	var thickness = erase_thickness if current_mode == DrawMode.ERASE else draw_thickness
	var color = Color(0, 0, 0, 0) if current_mode == DrawMode.ERASE else Color.BLACK
	
	var half_thickness = int(thickness / 2.0)
	for dx in range(-half_thickness, half_thickness + 1):
		for dy in range(-half_thickness, half_thickness + 1):
			var px = x + dx
			var py = y + dy
			if px >= 0 and px < drawing_image.get_width() and py >= 0 and py < drawing_image.get_height():
				drawing_image.set_pixel(px, py, color)
				
func set_message(message: String) -> void:
	var lines = _message_into_lines(message)
	line1.text = lines[0] if lines.size() > 0 else ""
	line2.text = lines[1] if lines.size() > 1 else ""
	line3.text = lines[2] if lines.size() > 2 else ""
	line4.text = lines[3] if lines.size() > 3 else ""
	line5.text = lines[4] if lines.size() > 4 else ""
	
func set_username(user: String) -> void:
	username.text = user
	await get_tree().process_frame
	var username_text_size: float = username.get_theme_font("font").get_string_size(username.text, HORIZONTAL_ALIGNMENT_LEFT, -1, username.get_theme_font_size("font_size")).x
	username_bg.size.x = username_text_size + 16
	
	if !line1:
		return
	line1.position.x = username_bg.position.x + username_bg.size.x + 4
	
func set_note(user: String, message: String) -> void:
	set_username(user)
	set_message(message)
	
func _message_into_lines(message: String) -> Array[String]:
	var lines: Array[String] = []
	var remaining = message
	var is_first_line = true
	
	while remaining.length() > 0 and lines.size() < 5:
		var max_chars = 36
		if is_first_line:
			max_chars -= username.text.length() + 6 # users can only have 16 chars max
		
		if remaining.length() <= max_chars:
			lines.append(remaining)
			break
		else:
			var break_at = max_chars
			# break at spaces if found
			var space_pos = remaining.substr(0, max_chars + 1).rfind(" ")
			if space_pos > 0:
				break_at = space_pos
			lines.append(remaining.substr(0, break_at).strip_edges())
			remaining = remaining.substr(break_at).strip_edges()
			is_first_line = false
	return lines
	
func set_accent(color: Color) -> void:
	card_mask.modulate = color
	username_bg.modulate = color
	username_bg.modulate.darkened(0.2)
	
func get_drawing_rle() -> PackedByteArray:	
	var result = PackedByteArray()
	var first_pixel = drawing_image.get_pixel(0, 0)
	var current_bit = 0 if first_pixel.a < 0.5 else 1
	var count = 0
	
	for row in range(drawing_image.get_height()):
		for pixel in range(drawing_image.get_width()):
			var img_pixel = drawing_image.get_pixel(pixel, row)
			var bit = 0 if img_pixel.a < 0.5 else 1
			
			if bit == current_bit:
				count += 1
				if count == 255:  
					result.append(count)
					result.append(current_bit)
					count = 0 
			else:
				if count > 0:
					result.append(count)
					result.append(current_bit)
				current_bit = bit
				count = 1
				
	if count > 0:
		result.append(count)
		result.append(current_bit)
			
	return result
func from_rle(rle_data: PackedByteArray):
	if rle_data.size() == 0:
		return 
	drawing_image.fill(Color(0, 0, 0, 0))
	
	var pixel = 0
	var row = 0
	var canvas_y = drawing_image.get_height()
	var canvas_x = drawing_image.get_width()
	
	for i in range(0, rle_data.size(), 2):
		if i + 1 >= rle_data.size():
			break
		var count = int(rle_data[i])
		var value = int(rle_data[i+1])
		var color = Color(0,0,0,0) if value == 0 else Color.BLACK
		# for all pixels
		for _i in range(count):
			# hit row end
			drawing_image.set_pixel(pixel, row, color)
			pixel += 1
			if pixel >= canvas_x:
				row += 1
				pixel = 0
				if row >= canvas_y:
					break
					
	drawing_layer.texture = ImageTexture.create_from_image(drawing_image)

func clear_drawing():
	drawing_image.fill(Color(0, 0, 0, 0)) 
	drawing_layer.texture = ImageTexture.create_from_image(drawing_image)
		
func clear():
	clear_drawing()
	set_message("")
	
func is_drawing_empty() -> bool:
	for row in range(drawing_image.get_height()):
		for pixel in range(drawing_image.get_width()):
			var img_pixel = drawing_image.get_pixel(pixel, row)
			if img_pixel.a > 0.5:
				return false
	return true
	
func set_both_thickness(thickness: float) -> void:
	draw_thickness = thickness
	erase_thickness = thickness
