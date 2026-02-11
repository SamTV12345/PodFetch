import { useState, useEffect, useCallback } from 'react';
import { syncService } from '@/store/syncService';

/**
 * Hook um den Online-Status des Geräts zu verfolgen
 */
export function useNetworkStatus() {
    const [isOnline, setIsOnline] = useState<boolean>(true);
    const [isChecking, setIsChecking] = useState<boolean>(true);

    useEffect(() => {
        let mounted = true;

        syncService.isOnline().then((online) => {
            if (mounted) {
                setIsOnline(online);
                setIsChecking(false);
            }
        });

        // Subscribe to changes
        const unsubscribe = syncService.subscribeToOnlineStatus((online) => {
            if (mounted) {
                setIsOnline(online);
            }
        });

        return () => {
            mounted = false;
            unsubscribe();
        };
    }, []);

    const refresh = useCallback(async () => {
        setIsChecking(true);
        const online = await syncService.isOnline();
        setIsOnline(online);
        setIsChecking(false);
        return online;
    }, []);

    return { isOnline, isChecking, refresh };
}
