extends Node


var in_progress_transition = false
var _scene_to_load: String
var _scene_to_unload: Node

var loading_screen: LoadingScreen
var loading_screen_scene = preload("res://scenes/loading_screen.tscn")

var resource_timer_ping: Timer
var resource_loops_icon: int = 0

signal scene_loaded(scene: Node)

func _ready() -> void:
	scene_loaded.connect(_on_scene_loaded)

func swap_scene(scene_to_load: String, scene_to_unload: Node = null) -> void:
	if in_progress_transition:
		push_warning("transition ignored because of ongoing transition ")
		return
		
	in_progress_transition = true
	_scene_to_load = scene_to_load
	_scene_to_unload = scene_to_unload
	
	loading_screen = loading_screen_scene.instantiate() as LoadingScreen
	get_tree().root.add_child(loading_screen)
	loading_screen.start_transition()
	load_scene()

func load_scene() -> void:
	print("starting loading")
	ResourceLoader.load_threaded_request(_scene_to_load)
	resource_timer_ping = Timer.new()
	resource_timer_ping.wait_time = 0.1
	resource_timer_ping.timeout.connect(_on_resource_timer_ping)

	get_tree().root.add_child(resource_timer_ping)
	resource_timer_ping.start()
	
func _on_resource_timer_ping() -> void:
	print("loader ping")
	var resource = ResourceLoader.load_threaded_get_status(_scene_to_load)
	
	if resource == ResourceLoader.THREAD_LOAD_IN_PROGRESS:
		resource_loops_icon += 1
		
		if resource_loops_icon > 10:
			loading_screen.add_loading_icon()
		return
		
	if resource == ResourceLoader.THREAD_LOAD_LOADED:
		print("scene finalized loading")
		resource_timer_ping.stop()
		resource_timer_ping.queue_free()
		scene_loaded.emit(ResourceLoader.load_threaded_get(_scene_to_load).instantiate())
		
func _on_scene_loaded(scene: Node) -> void:
	await get_tree().create_timer(1).timeout
	print("cleaning up old scene and transition")
	get_tree().root.add_child(scene)
	
	if _scene_to_unload != null:
		if _scene_to_unload != get_tree().root: 
			print("scene to unload is not the root tree")
			_scene_to_unload.queue_free()

	await loading_screen.end_transition()
	loading_screen.queue_free()
	
	in_progress_transition = false
	
