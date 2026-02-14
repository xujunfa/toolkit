import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Play, Square, RefreshCw } from 'lucide-react';
import type { ZenmuxUsageData } from '@/core/ipc.generated';
import {
  useZenmuxConfig,
  useSetZenmuxConfig,
  useGetZenmuxUsage,
  useStartZenmuxPolling,
  useStopZenmuxPolling,
} from '@/modules/zenmux';
import { Button } from '@/components/ui/button';

export function ZenMuxQuotaPage() {
  const [cookieInput, setCookieInput] = useState('');
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const [usageData, setUsageData] = useState<ZenmuxUsageData | null>(null);
  const [pollingActive, setPollingActive] = useState(false);

  const configQuery = useZenmuxConfig();
  const setConfigMutation = useSetZenmuxConfig();
  const fetchUsageMutation = useGetZenmuxUsage();
  const startPollingMutation = useStartZenmuxPolling();
  const stopPollingMutation = useStopZenmuxPolling();

  // Listen for real-time usage updates from polling
  useEffect(() => {
    const unlisten = listen<ZenmuxUsageData>('zenmux-usage-updated', (event) => {
      setUsageData(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  function handleSaveCookie() {
    setConfigMutation.mutate(cookieInput, {
      onSuccess: () => {
        setSaveMsg('Cookie saved');
        setTimeout(() => setSaveMsg(null), 3000);
      },
      onError: (err) => {
        setSaveMsg(`Error: ${err}`);
      },
    });
  }

  function handleFetchNow() {
    fetchUsageMutation.mutate(undefined, {
      onSuccess: (data) => {
        setUsageData(data);
      },
    });
  }

  function handleStartPolling() {
    startPollingMutation.mutate(undefined, {
      onSuccess: () => setPollingActive(true),
    });
  }

  function handleStopPolling() {
    stopPollingMutation.mutate(undefined, {
      onSuccess: () => setPollingActive(false),
    });
  }

  const hasConfig = configQuery.data && configQuery.data.ctoken !== '';

  return (
    <div className="p-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">
          ZenMux Quota Monitor
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Monitor your ZenMux subscription quota in the menu bar.
        </p>
      </div>

      {/* Cookie Config */}
      <div className="mt-6 space-y-3">
        <h2 className="text-sm font-medium">Cookie Configuration</h2>
        <textarea
          className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
          rows={3}
          placeholder="ctoken=xxx; sessionId=xxx; sessionId.sig=xxx"
          value={cookieInput}
          onChange={(e) => setCookieInput(e.target.value)}
        />
        <div className="flex items-center gap-3">
          <Button
            onClick={handleSaveCookie}
            disabled={setConfigMutation.isPending || !cookieInput.trim()}
          >
            Save Cookie
          </Button>
          {saveMsg && (
            <span className="text-sm text-muted-foreground">{saveMsg}</span>
          )}
        </div>
        {hasConfig && (
          <p className="text-xs text-muted-foreground">
            Current ctoken: {configQuery.data!.ctoken.slice(0, 8)}...
            {' | '}Last updated: {configQuery.data!.updated_at}
          </p>
        )}
      </div>

      {/* Polling Controls */}
      <div className="mt-6 space-y-3">
        <h2 className="text-sm font-medium">Polling Controls</h2>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={handleStartPolling}
            disabled={startPollingMutation.isPending || !hasConfig}
          >
            <Play className="h-4 w-4" />
            Start Polling
          </Button>
          <Button
            variant="outline"
            onClick={handleStopPolling}
            disabled={stopPollingMutation.isPending}
          >
            <Square className="h-4 w-4" />
            Stop Polling
          </Button>
          <Button
            variant="outline"
            onClick={handleFetchNow}
            disabled={fetchUsageMutation.isPending || !hasConfig}
          >
            <RefreshCw
              className={`h-4 w-4 ${fetchUsageMutation.isPending ? 'animate-spin' : ''}`}
            />
            Fetch Now
          </Button>
        </div>
        {pollingActive && (
          <p className="text-xs text-muted-foreground">
            Polling active (60s interval)
          </p>
        )}
      </div>

      {/* Usage Status */}
      <div className="mt-6 space-y-3">
        <h2 className="text-sm font-medium">Quota Status</h2>
        {usageData ? (
          <div className="rounded-md border border-border p-4 space-y-3">
            <div className="flex items-center gap-3">
              <span className="text-sm font-medium">Tray Preview:</span>
              <code className="rounded bg-muted px-2 py-1 text-sm font-mono">
                {usageData.tray_text}
              </code>
            </div>
            <div className="text-xs text-muted-foreground">
              Last fetched: {usageData.fetched_at}
            </div>
            {usageData.items.length > 0 && (
              <div className="space-y-2">
                {usageData.items.map((item, i) => {
                  const remainingPct = Math.round(
                    (1 - item.used_rate) * 100,
                  );
                  return (
                    <div
                      key={i}
                      className="flex items-center gap-4 text-sm"
                    >
                      <span className="w-16 font-mono text-muted-foreground">
                        {item.period_type === 'hour_5'
                          ? '5-Hour'
                          : item.period_type === 'week'
                            ? 'Weekly'
                            : item.period_type}
                      </span>
                      <div className="flex-1">
                        <div className="h-2 rounded-full bg-muted">
                          <div
                            className="h-2 rounded-full bg-primary"
                            style={{ width: `${remainingPct}%` }}
                          />
                        </div>
                      </div>
                      <span className="w-12 text-right font-mono">
                        {remainingPct}%
                      </span>
                      <span className="text-xs text-muted-foreground">
                        resets {item.cycle_end_time}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">
            {fetchUsageMutation.isError
              ? `Error: ${fetchUsageMutation.error}`
              : 'No data yet. Click "Fetch Now" or start polling.'}
          </p>
        )}
      </div>
    </div>
  );
}
