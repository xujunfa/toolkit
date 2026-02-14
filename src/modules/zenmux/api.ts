import { typedInvoke } from '@/lib/tauri';

export function getZenmuxConfig() {
  return typedInvoke('get_zenmux_config', {});
}

export function setZenmuxConfig(cookie: string) {
  return typedInvoke('set_zenmux_config', { cookie });
}

export function getZenmuxUsage() {
  return typedInvoke('get_zenmux_usage', {});
}

export function startZenmuxPolling() {
  return typedInvoke('start_zenmux_polling', {});
}

export function stopZenmuxPolling() {
  return typedInvoke('stop_zenmux_polling', {});
}
