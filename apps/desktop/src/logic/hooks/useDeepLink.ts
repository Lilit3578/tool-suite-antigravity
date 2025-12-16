import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

export function useDeepLinkListener() {
    const [isVerifying, setIsVerifying] = useState(false);

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen<string>('auth-deep-link', (event) => {
                const token = event.payload;
                console.log(`Deep Link Received: [${token}]`);
                setIsVerifying(true);

                // Placeholder for verification logic
                setTimeout(() => {
                    setIsVerifying(false);
                    console.log('Verification placeholder complete');
                }, 2000);
            });
        };

        setupListener();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, []);

    return { isVerifying };
}
