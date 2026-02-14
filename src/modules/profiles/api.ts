import { typedInvoke } from '@/lib/tauri';
import type { CommandArg } from '@/core/ipc';

export function listProfiles() {
  return typedInvoke('list_profiles', {});
}

export function createProfile(input: CommandArg<'create_profile'>['input']) {
  return typedInvoke('create_profile', { input });
}

export function updateProfile(
  id: string,
  input: CommandArg<'update_profile'>['input'],
) {
  return typedInvoke('update_profile', { id, input });
}

export function deleteProfile(id: string) {
  return typedInvoke('delete_profile', { id });
}

export function syncProfilesToZshrc(useRealZshrc: boolean) {
  return typedInvoke('sync_profiles_to_zshrc', { useRealZshrc });
}
