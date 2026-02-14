import { describe, it, expect } from 'vitest';
import { COMMAND_NAMES } from './ipc.generated';

describe('ipc generated types', () => {
  it('should only include template command names', () => {
    expect(COMMAND_NAMES).toEqual([
      'ping',
      'get_app_info',
      'get_settings',
      'set_settings',
      'list_profiles',
      'create_profile',
      'update_profile',
      'delete_profile',
      'sync_profiles_to_zshrc',
      'get_zenmux_config',
      'set_zenmux_config',
      'get_zenmux_usage',
      'start_zenmux_polling',
      'stop_zenmux_polling',
      'update_tray_title',
    ]);
  });
});
