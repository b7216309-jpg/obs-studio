#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <obs-internal.h>

/* obs-hotkey.c uses this isolated core (renamed by CMake). Do not start OBS
 * or its platform input thread: these tests only load synthetic bindings. */
struct obs_core *hotkey_test_obs;

bool obs_hotkeys_platform_is_pressed(obs_hotkeys_platform_t *context, obs_key_t key)
{
	UNUSED_PARAMETER(context);
	UNUSED_PARAMETER(key);
	return false;
}

static int setup(void **state)
{
	UNUSED_PARAMETER(state);
	hotkey_test_obs = bzalloc(sizeof(*hotkey_test_obs));
	assert_int_equal(pthread_mutex_init_recursive(&hotkey_test_obs->hotkeys.mutex), 0);
	hotkey_test_obs->hotkeys.signals = signal_handler_create();
	static const char *signals[] = {"void hotkey_register(ptr key)", "void hotkey_unregister(ptr key)",
					"void hotkey_bindings_changed(ptr key)", NULL};
	assert_true(signal_handler_add_array(hotkey_test_obs->hotkeys.signals, signals));
	return 0;
}

static int teardown(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkeys_free();
	signal_handler_destroy(hotkey_test_obs->hotkeys.signals);
	pthread_mutex_destroy(&hotkey_test_obs->hotkeys.mutex);
	bfree(hotkey_test_obs);
	hotkey_test_obs = NULL;
	return 0;
}

static void hotkey_callback(void *data, obs_hotkey_id id, obs_hotkey_t *hotkey, bool pressed)
{
	UNUSED_PARAMETER(id);
	UNUSED_PARAMETER(hotkey);
	if (data && pressed)
		++*(size_t *)data;
}

static obs_hotkey_id register_hotkey(void)
{
	obs_hotkey_id id = obs_hotkey_register_frontend("test", "Test", hotkey_callback, NULL);
	assert_int_not_equal(id, OBS_INVALID_HOTKEY_ID);
	return id;
}

static obs_data_array_t *bindings_from_json(const char *json)
{
	obs_data_t *data = obs_data_create_from_json(json);
	assert_non_null(data);
	obs_data_array_t *bindings = obs_data_get_array(data, "bindings");
	obs_data_release(data);
	assert_non_null(bindings);
	return bindings;
}

static void load_json(obs_hotkey_id id, const char *json)
{
	obs_data_array_t *bindings = bindings_from_json(json);
	obs_hotkey_load(id, bindings);
	obs_data_array_release(bindings);
}

static size_t saved_binding_count(obs_hotkey_id id)
{
	obs_data_array_t *bindings = obs_hotkey_save(id);
	assert_non_null(bindings);
	size_t count = obs_data_array_count(bindings);
	obs_data_array_release(bindings);
	return count;
}

static const char *blocked_json = "{\"bindings\":["
				  "{\"key\":\"OBS_KEY_MOUSE1\"},"
				  "{\"key\":\"OBS_KEY_MOUSE2\"},"
				  "{\"key\":\"OBS_KEY_MOUSE1\",\"shift\":true,\"control\":true},"
				  "{\"key\":\"OBS_KEY_MOUSE2\",\"alt\":true,\"command\":true}]}";

static void json_rejects_primary_mouse_buttons(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_id id = register_hotkey();
	load_json(id, blocked_json);
	assert_int_equal(saved_binding_count(id), 0);
}

static void api_rejects_primary_mouse_buttons(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_id id = register_hotkey();
	const uint32_t flags[] = {INTERACT_SHIFT_KEY, INTERACT_CONTROL_KEY, INTERACT_ALT_KEY, INTERACT_COMMAND_KEY};
	for (unsigned mask = 0; mask < 16; mask++) {
		uint32_t modifiers = 0;
		for (size_t i = 0; i < 4; i++)
			if (mask & (1U << i))
				modifiers |= flags[i];
		obs_key_combination_t bindings[] = {{modifiers, OBS_KEY_MOUSE1}, {modifiers, OBS_KEY_MOUSE2}};
		obs_hotkey_load_bindings(id, bindings, 2);
		assert_int_equal(saved_binding_count(id), 0);
	}
}

static const obs_key_combination_t allowed[] = {
	{INTERACT_CONTROL_KEY, OBS_KEY_A},  {0, OBS_KEY_MOUSE3},
	{INTERACT_ALT_KEY, OBS_KEY_MOUSE4}, {0, OBS_KEY_MOUSE27},
	{INTERACT_SHIFT_KEY, OBS_KEY_NONE},
};

static bool check_allowed_binding(void *data, size_t idx, obs_hotkey_binding_t *binding)
{
	size_t *count = data;
	assert_true(idx < sizeof(allowed) / sizeof(allowed[0]));
	obs_key_combination_t combo = obs_hotkey_binding_get_key_combination(binding);
	assert_int_equal(combo.key, allowed[idx].key);
	assert_int_equal(combo.modifiers, allowed[idx].modifiers);
	++*count;
	return true;
}

static void assert_allowed_bindings(obs_hotkey_id id)
{
	assert_int_equal(saved_binding_count(id), sizeof(allowed) / sizeof(allowed[0]));
	size_t count = 0;
	obs_enum_hotkey_bindings(check_allowed_binding, &count);
	assert_int_equal(count, sizeof(allowed) / sizeof(allowed[0]));
}

static void json_preserves_allowed_bindings(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_id id = register_hotkey();
	load_json(id, "{\"bindings\":["
		      "{\"key\":\"OBS_KEY_MOUSE1\"},"
		      "{\"key\":\"OBS_KEY_A\",\"control\":true},"
		      "{\"key\":\"OBS_KEY_MOUSE3\"},"
		      "{\"key\":\"OBS_KEY_MOUSE2\",\"shift\":true},"
		      "{\"key\":\"OBS_KEY_MOUSE4\",\"alt\":true},"
		      "{\"key\":\"OBS_KEY_MOUSE27\"},"
		      "{\"key\":\"OBS_KEY_NONE\",\"shift\":true}]}");
	assert_allowed_bindings(id);

	/* Saving and reloading must not reintroduce the discarded bindings. */
	obs_data_array_t *saved = obs_hotkey_save(id);
	obs_hotkey_load(id, saved);
	obs_data_array_release(saved);
	assert_allowed_bindings(id);
}

static void api_preserves_allowed_bindings(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_id id = register_hotkey();
	obs_key_combination_t bindings[] = {
		{0, OBS_KEY_MOUSE2}, allowed[0], allowed[1], {INTERACT_CONTROL_KEY, OBS_KEY_MOUSE1},
		allowed[2],          allowed[3], allowed[4],
	};
	obs_hotkey_load_bindings(id, bindings, sizeof(bindings) / sizeof(bindings[0]));
	assert_allowed_bindings(id);
}

static void rejected_bindings_replace_old_bindings(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_id id = register_hotkey();
	obs_key_combination_t valid = {0, OBS_KEY_A};
	obs_key_combination_t blocked = {0, OBS_KEY_MOUSE1};
	obs_hotkey_load_bindings(id, &valid, 1);
	assert_int_equal(saved_binding_count(id), 1);
	obs_hotkey_load_bindings(id, &blocked, 1);
	assert_int_equal(saved_binding_count(id), 0);

	obs_hotkey_load_bindings(id, &valid, 1);
	load_json(id, blocked_json);
	assert_int_equal(saved_binding_count(id), 0);
}

static void pair_load_rejects_primary_mouse_buttons(void **state)
{
	UNUSED_PARAMETER(state);
	obs_hotkey_pair_id pair =
		obs_hotkey_pair_register_frontend("first", "First", "second", "Second", NULL, NULL, NULL, NULL);
	assert_int_not_equal(pair, OBS_INVALID_HOTKEY_PAIR_ID);
	obs_data_array_t *input = bindings_from_json(blocked_json);
	obs_hotkey_pair_load(pair, input, input);
	obs_data_array_release(input);
	obs_data_array_t *first = NULL, *second = NULL;
	obs_hotkey_pair_save(pair, &first, &second);
	assert_non_null(first);
	assert_non_null(second);
	size_t first_count = obs_data_array_count(first);
	size_t second_count = obs_data_array_count(second);
	obs_data_array_release(first);
	obs_data_array_release(second);
	assert_int_equal(first_count, 0);
	assert_int_equal(second_count, 0);
}

static void primary_mouse_buttons_do_not_trigger_callbacks(void **state)
{
	UNUSED_PARAMETER(state);
	size_t presses = 0;
	obs_hotkey_id id = obs_hotkey_register_frontend("click", "Click", hotkey_callback, &presses);
	load_json(id, blocked_json);
	obs_hotkey_inject_event((obs_key_combination_t){0, OBS_KEY_MOUSE1}, true);
	obs_hotkey_inject_event((obs_key_combination_t){0, OBS_KEY_MOUSE2}, true);
	assert_int_equal(presses, 0);

	obs_key_combination_t middle = {0, OBS_KEY_MOUSE3};
	obs_hotkey_load_bindings(id, &middle, 1);
	obs_hotkey_inject_event(middle, true);
	assert_int_equal(presses, 1);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test_setup_teardown(json_rejects_primary_mouse_buttons, setup, teardown),
		cmocka_unit_test_setup_teardown(api_rejects_primary_mouse_buttons, setup, teardown),
		cmocka_unit_test_setup_teardown(json_preserves_allowed_bindings, setup, teardown),
		cmocka_unit_test_setup_teardown(api_preserves_allowed_bindings, setup, teardown),
		cmocka_unit_test_setup_teardown(rejected_bindings_replace_old_bindings, setup, teardown),
		cmocka_unit_test_setup_teardown(pair_load_rejects_primary_mouse_buttons, setup, teardown),
		cmocka_unit_test_setup_teardown(primary_mouse_buttons_do_not_trigger_callbacks, setup, teardown),
	};
	return cmocka_run_group_tests(tests, NULL, NULL);
}
