import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getZenmuxConfig,
  setZenmuxConfig,
  getZenmuxUsage,
  startZenmuxPolling,
  stopZenmuxPolling,
} from './api';

const ZENMUX_CONFIG_KEY = ['zenmux-config'] as const;

export function useZenmuxConfig() {
  return useQuery({
    queryKey: ZENMUX_CONFIG_KEY,
    queryFn: getZenmuxConfig,
  });
}

export function useSetZenmuxConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (cookie: string) => setZenmuxConfig(cookie),
    onSuccess: () => qc.invalidateQueries({ queryKey: ZENMUX_CONFIG_KEY }),
  });
}

export function useGetZenmuxUsage() {
  return useMutation({
    mutationFn: () => getZenmuxUsage(),
  });
}

export function useStartZenmuxPolling() {
  return useMutation({
    mutationFn: () => startZenmuxPolling(),
  });
}

export function useStopZenmuxPolling() {
  return useMutation({
    mutationFn: () => stopZenmuxPolling(),
  });
}
