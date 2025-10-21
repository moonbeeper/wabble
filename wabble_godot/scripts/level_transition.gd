extends Control
class_name LoadingScreen

@onready var color: ColorRect= $ColorRect
@onready var animation_player: AnimationPlayer = $AnimationPlayer
@onready var loading: TextureRect = $Loading

var show_icon = false
var rot_time_acum = 0.0;

func _ready() -> void:
	color.visible = false
	loading.modulate = Color(0,0,0,0)

func start_transition():
	color.visible = true
	animation_player.play("to_right")
	await animation_player.animation_finished

func end_transition():
	animation_player.play("from_right")
	await animation_player.animation_finished
	color.visible = false
	
func add_loading_icon():
	var tween = get_tree().create_tween()
	tween.tween_property(loading, "modulate", Color(1,1,1,1),0.7).set_trans(Tween.TRANS_SINE)
	await tween.finished
	show_icon = true
	
func _process(delta: float) -> void:
	if show_icon:
		rot_time_acum += delta
		if rot_time_acum >= 0.5:
				rot_time_acum -= 0.5
				loading.rotation -= TAU / 12
