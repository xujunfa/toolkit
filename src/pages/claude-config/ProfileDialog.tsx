import { useEffect, useState } from 'react';
import { Plus, X } from 'lucide-react';
import type { EnvVar, ProfileResponse } from '@/core/ipc.generated';
import { useCreateProfile, useUpdateProfile } from '@/modules/profiles/queries';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';

interface ProfileDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  profile?: ProfileResponse;
}

export function ProfileDialog({
  open,
  onOpenChange,
  profile,
}: ProfileDialogProps) {
  const isEditMode = !!profile;

  const [name, setName] = useState('');
  const [alias, setAlias] = useState('');
  const [envVars, setEnvVars] = useState<EnvVar[]>([]);

  const createMutation = useCreateProfile();
  const updateMutation = useUpdateProfile();

  const isPending = createMutation.isPending || updateMutation.isPending;

  useEffect(() => {
    if (open) {
      if (profile) {
        setName(profile.name);
        setAlias(profile.alias);
        setEnvVars(profile.env_vars.map((v) => ({ ...v })));
      } else {
        setName('');
        setAlias('');
        setEnvVars([]);
      }
    }
  }, [open, profile]);

  function addEnvVar() {
    setEnvVars((prev) => [...prev, { key: '', value: '' }]);
  }

  function removeEnvVar(index: number) {
    setEnvVars((prev) => prev.filter((_, i) => i !== index));
  }

  function updateEnvVar(index: number, field: keyof EnvVar, value: string) {
    setEnvVars((prev) =>
      prev.map((v, i) => (i === index ? { ...v, [field]: value } : v))
    );
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    const trimmedName = name.trim();
    const trimmedAlias = alias.trim();
    if (!trimmedName) return;

    const cleanedEnvVars = envVars.filter(
      (v) => v.key.trim() !== '' || v.value.trim() !== ''
    );

    if (isEditMode) {
      await updateMutation.mutateAsync({
        id: profile.id,
        input: {
          name: trimmedName,
          alias: trimmedAlias,
          env_vars: cleanedEnvVars,
        },
      });
    } else {
      await createMutation.mutateAsync({
        name: trimmedName,
        alias: trimmedAlias,
        env_vars: cleanedEnvVars,
      });
    }

    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>
              {isEditMode ? 'Edit Profile' : 'Create Profile'}
            </DialogTitle>
            <DialogDescription>
              {isEditMode
                ? 'Update the profile configuration.'
                : 'Add a new Claude Code launch profile.'}
            </DialogDescription>
          </DialogHeader>

          <div className="mt-4 grid gap-4">
            <div className="grid gap-2">
              <label htmlFor="profile-name" className="text-sm font-medium">
                Name
              </label>
              <Input
                id="profile-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My Profile"
                required
              />
            </div>

            <div className="grid gap-2">
              <label htmlFor="profile-alias" className="text-sm font-medium">
                Alias
              </label>
              <Input
                id="profile-alias"
                value={alias}
                onChange={(e) => setAlias(e.target.value)}
                placeholder="e.g. ccleo"
              />
            </div>

            <div className="grid gap-2">
              <label className="text-sm font-medium">
                Environment Variables
              </label>

              {envVars.map((envVar, index) => (
                <div key={index} className="flex items-center gap-2">
                  <Input
                    value={envVar.key}
                    onChange={(e) => updateEnvVar(index, 'key', e.target.value)}
                    placeholder="KEY"
                    className="flex-1"
                  />
                  <Input
                    value={envVar.value}
                    onChange={(e) =>
                      updateEnvVar(index, 'value', e.target.value)
                    }
                    placeholder="value"
                    className="flex-1"
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-xs"
                    onClick={() => removeEnvVar(index)}
                    aria-label="Remove variable"
                  >
                    <X />
                  </Button>
                </div>
              ))}

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={addEnvVar}
                className="w-fit"
              >
                <Plus />
                Add Variable
              </Button>
            </div>
          </div>

          <DialogFooter className="mt-6">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isPending}>
              {isEditMode ? 'Save' : 'Create'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
