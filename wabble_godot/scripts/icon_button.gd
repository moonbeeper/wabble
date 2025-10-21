@tool
extends Button

var pop_tween: Tween
var hover_tween: Tween

@export var iconTex: TextureRect
@export var iconTexture: Texture

func _ready() -> void:
	if iconTexture:
		iconTex.texture = iconTexture

func _on_pressed() -> void:
	if pop_tween and pop_tween.is_running():
		pop_tween.kill()
	pop_tween = get_tree().create_tween().set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	pop_tween.tween_property(self, "modulate", Color(1.5, 1.5, 1.5, 1.0), 0.1).from(Color(1,1,1,1))
	pop_tween.chain().tween_property(self, "modulate", Color(1, 1, 1, 1), 0.2).from(Color(1.5,1.5,1.5,1))
