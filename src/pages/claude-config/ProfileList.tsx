import { Pencil, Trash2 } from 'lucide-react';
import type { ProfileResponse } from '@/core/ipc.generated';
import { useProfiles } from '@/modules/profiles/queries';
import { Button } from '@/components/ui/button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

function maskSensitiveValue(key: string, value: string): string {
  const isSensitiveKey = /key|token|secret/i.test(key);
  const isSensitiveValue = value.startsWith('sk-');

  if (isSensitiveKey || isSensitiveValue) {
    if (value.length <= 4) return '****';
    return value.slice(0, 4) + '****';
  }

  return value;
}

interface ProfileListProps {
  onEdit: (profile: ProfileResponse) => void;
  onDelete: (profile: ProfileResponse) => void;
}

export function ProfileList({ onEdit, onDelete }: ProfileListProps) {
  const { data: profiles, isLoading } = useProfiles();

  if (isLoading) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        Loading...
      </p>
    );
  }

  if (!profiles || profiles.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        No profiles yet. Create one to get started.
      </p>
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Alias</TableHead>
          <TableHead>Env Vars</TableHead>
          <TableHead className="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {profiles.map((profile) => (
          <TableRow key={profile.id}>
            <TableCell className="font-medium">{profile.name}</TableCell>
            <TableCell className="text-muted-foreground">
              {profile.alias}
            </TableCell>
            <TableCell>
              {profile.env_vars.length === 0 ? (
                <span className="text-muted-foreground">None</span>
              ) : (
                <span
                  className="cursor-default"
                  title={profile.env_vars
                    .map(
                      (v) =>
                        `${v.key}=${maskSensitiveValue(v.key, v.value)}`
                    )
                    .join('\n')}
                >
                  {profile.env_vars.length}{' '}
                  {profile.env_vars.length === 1 ? 'variable' : 'variables'}
                </span>
              )}
            </TableCell>
            <TableCell className="text-right">
              <div className="flex items-center justify-end gap-1">
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => onEdit(profile)}
                  aria-label={`Edit ${profile.name}`}
                >
                  <Pencil />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => onDelete(profile)}
                  aria-label={`Delete ${profile.name}`}
                >
                  <Trash2 />
                </Button>
              </div>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
