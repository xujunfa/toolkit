import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listProfiles,
  createProfile,
  updateProfile,
  deleteProfile,
  syncProfilesToZshrc,
} from './api';
import type { CommandArg } from '@/core/ipc';

const PROFILES_KEY = ['profiles'] as const;

export function useProfiles() {
  return useQuery({
    queryKey: PROFILES_KEY,
    queryFn: listProfiles,
  });
}

export function useCreateProfile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CommandArg<'create_profile'>['input']) =>
      createProfile(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: PROFILES_KEY }),
  });
}

export function useUpdateProfile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      input,
    }: {
      id: string;
      input: CommandArg<'update_profile'>['input'];
    }) => updateProfile(id, input),
    onSuccess: () => qc.invalidateQueries({ queryKey: PROFILES_KEY }),
  });
}

export function useDeleteProfile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteProfile(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: PROFILES_KEY }),
  });
}

export function useSyncToZshrc() {
  return useMutation({
    mutationFn: (useRealZshrc: boolean) => syncProfilesToZshrc(useRealZshrc),
  });
}
