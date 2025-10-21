@tool
extends Panel
class_name PanelHeader

@export var _titleLabel: RichTextLabel
@export var titleText: String = "Hello world":
	set(value):
		titleText = value
		needs_update = true
	get:
		return titleText
@export var titleColor: Color = Color(0,0,0,1)
@export var _ditherTex: TextureRect
@export var is_bottom: bool = false
@export var background_color: Color = Color(0.9, 0.0, 1.0, 1.0)

var needs_update: bool = true

var wow_tween: Tween
var original_position: Vector2 = Vector2(0,0)

func _ready() -> void:
	var panel_style = self.get_theme_stylebox("panel").duplicate() as StyleBoxFlat
	if is_bottom:
		_ditherTex.rotation_degrees = 180
		panel_style.border_width_bottom = 0
		panel_style.border_width_top = 4
	panel_style.bg_color = background_color
	self.add_theme_stylebox_override("panel", panel_style)
	_titleLabel.text = titleText
	_titleLabel.add_theme_color_override("default_color", titleColor)

func _process(_delta: float) -> void:
	if !needs_update: return
	_titleLabel.text = titleText

func tween_hide() -> void:
	if wow_tween && wow_tween.is_running():
		wow_tween.kill()
		
	wow_tween = get_tree().create_tween()
	original_position = self.position
	var padding = 20
	var to = self.size.y + padding
	if is_bottom:
		to += self.position.y
	else:
		to = -to
	wow_tween.tween_property(self, "position", Vector2(0, to), .5)
	await wow_tween.finished
	self.visible = false
	
func tween_show() -> void:
	if wow_tween && wow_tween.is_running():
		wow_tween.kill()
	self.visible = true
	self.modulate = Color(0,0,0,0)
	wow_tween = get_tree().create_tween()
	wow_tween.tween_property(self, "position", original_position, .5).from_current()
	wow_tween.parallel().tween_property(self, "modulate", Color(1,1,1,1), .1)
	await wow_tween.finished
