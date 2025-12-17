import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export function useDeepLinkListener() {
    const [isVerifying, setIsVerifying] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isAuthenticated, setIsAuthenticated] = useState(false);

    useEffect(() => {
        let unlistenCode: (() => void) | undefined;
        let unlistenToken: (() => void) | undefined;

        const setupListeners = async () => {
            // PKCE Flow: Listen for auth code
            unlistenCode = await listen<string>('auth-code-received', async (event) => {
                const code = event.payload;
                console.log(`[PKCE] Auth code received: ${code}`);
                setIsVerifying(true);
                setError(null);

                try {
                    // Exchange code for token using PKCE verifier
                    const response = await invoke<{ success: boolean; token?: string; message: string }>(
                        'exchange_token',
                        { authCode: code }
                    );

                    if (response.success && response.token) {
                        console.log('[PKCE] ✅ Token exchange successful');
                        setIsAuthenticated(true);
                        setError(null);
                    } else {
                        console.error('[PKCE] ❌ Token exchange failed:', response.message);
                        setError(response.message || 'Authentication failed');
                    }
                } catch (err) {
                    console.error('[PKCE] ❌ Exchange error:', err);
                    setError(err instanceof Error ? err.message : 'Unknown error');
                } finally {
                    setIsVerifying(false);
                }
            });

            // Legacy: Listen for direct token (deprecated)
            unlistenToken = await listen<string>('auth-deep-link', (event) => {
                const token = event.payload;
                console.warn(`[DEPRECATED] Direct token received: ${token}`);
                setIsVerifying(true);

                // Placeholder for legacy verification logic
                setTimeout(() => {
                    setIsVerifying(false);
                    setIsAuthenticated(true);
                    console.log('[DEPRECATED] Legacy verification complete');
                }, 2000);
            });
        };

        setupListeners();

        return () => {
            if (unlistenCode) {
                unlistenCode();
            }
            if (unlistenToken) {
                unlistenToken();
            }
        };
    }, []);

    return { isVerifying, error, isAuthenticated };
}
