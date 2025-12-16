
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export function useDeepLinkAuth() {
    useEffect(() => {
        // Handler for performing handshake
        const handleHandshake = async (otp: string) => {
            try {
                console.log('Initiating handshake with OTP:', otp);
                const res = await invoke<{ success: boolean, token?: string }>('perform_handshake', { otp });
                if (res.success && res.token) {
                    console.log('Handshake successful!');
                    // Here you might update global state, e.g. isAuthenticated = true
                    // preventing duplication if already handled by keyring check on startup
                    // For now, we'll just log it.
                }
            } catch (error) {
                console.error('Handshake failed:', error);
            }
        };

        // Listener 1: Cold Start / Standard Deep Link (from lib.rs on_open_url)
        const unlistenOtp = listen<string>('auth-otp-received', (event) => {
            handleHandshake(event.payload);
        });

        // Listener 2: Single Instance (Already running, second instance launched)
        const unlistenSingleInstance = listen<string[]>('deep-link://new-url', (event) => {
            const args = event.payload;
            // Find argument starting with prodwidgets://
            const deepLink = args.find(arg => arg.startsWith('prodwidgets://'));
            if (deepLink) {
                try {
                    const url = new URL(deepLink);
                    const otp = url.searchParams.get('otp');
                    if (otp) {
                        handleHandshake(otp);
                    }
                } catch (e) {
                    console.error('Failed to parse deep link URL:', e);
                }
            }
        });

        return () => {
            unlistenOtp.then(fn => fn());
            unlistenSingleInstance.then(fn => fn());
        };
    }, []);
}
