import { useState } from 'react';
import { Plus, RefreshCw } from 'lucide-react';
import type { ProfileResponse } from '@/core/ipc.generated';
import { useDeleteProfile, useSyncToZshrc } from '@/modules/profiles/queries';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { ProfileList } from '@/pages/claude-config/ProfileList';
import { ProfileDialog } from '@/pages/claude-config/ProfileDialog';
import { DeleteConfirmDialog } from '@/pages/claude-config/DeleteConfirmDialog';

export function ClaudeConfigPage() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingProfile, setEditingProfile] = useState<
    ProfileResponse | undefined
  >(undefined);
  const [deleteTarget, setDeleteTarget] = useState<
    ProfileResponse | undefined
  >(undefined);

  const deleteMutation = useDeleteProfile();
  const syncMutation = useSyncToZshrc();
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [syncTargetPath, setSyncTargetPath] = useState<string | null>(null);
  const [lastSyncUsedRealZshrc, setLastSyncUsedRealZshrc] = useState(false);
  const [useRealZshrc, setUseRealZshrc] = useState(false);

  function handleEdit(profile: ProfileResponse) {
    setEditingProfile(profile);
    setDialogOpen(true);
  }

  function handleCreate() {
    setEditingProfile(undefined);
    setDialogOpen(true);
  }

  function handleDelete(profile: ProfileResponse) {
    setDeleteTarget(profile);
  }

  function handleConfirmDelete() {
    if (deleteTarget) {
      deleteMutation.mutate(deleteTarget.id);
      setDeleteTarget(undefined);
    }
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">
            Claude Code Profiles
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Manage your Claude Code launch configurations.
          </p>
        </div>
        <div className="flex gap-2">
          <label className="flex items-center gap-2 rounded-md border border-border px-3 text-sm text-muted-foreground">
            <Checkbox
              checked={useRealZshrc}
              onCheckedChange={(checked) => setUseRealZshrc(checked === true)}
              aria-label="Sync to real ~/.zshrc"
            />
            Sync to real ~/.zshrc
          </label>
          <Button
            variant="outline"
            onClick={() =>
              syncMutation.mutate(useRealZshrc, {
                onSuccess: (result) => {
                  setSyncMessage(result.message);
                  setSyncTargetPath(result.target_path);
                  setLastSyncUsedRealZshrc(result.is_real_zshrc);
                  setTimeout(() => setSyncMessage(null), 5000);
                },
              })
            }
            disabled={syncMutation.isPending}
          >
            <RefreshCw className={syncMutation.isPending ? 'animate-spin' : ''} />
            Sync to .zshrc
          </Button>
          <Button onClick={handleCreate}>
            <Plus />
            Add Profile
          </Button>
        </div>
      </div>

      {syncMessage && (
        <div className="mt-4 rounded-md border border-border bg-muted px-4 py-2 text-sm text-muted-foreground">
          {syncMessage}
          {lastSyncUsedRealZshrc ? (
            <> — run <code className="font-mono">source ~/.zshrc</code> to apply.</>
          ) : (
            <>
              {' '}
              — mock file updated at{' '}
              <code className="font-mono">{syncTargetPath}</code>.
            </>
          )}
        </div>
      )}

      <div className="mt-6">
        <ProfileList onEdit={handleEdit} onDelete={handleDelete} />
      </div>

      <ProfileDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        profile={editingProfile}
      />

      <DeleteConfirmDialog
        open={!!deleteTarget}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(undefined);
        }}
        profileName={deleteTarget?.name ?? ''}
        onConfirm={handleConfirmDelete}
      />
    </div>
  );
}
