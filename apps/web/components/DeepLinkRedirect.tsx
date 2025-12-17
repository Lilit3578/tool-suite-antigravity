'use client';

import { useEffect } from 'react';

interface DeepLinkRedirectProps {
    deepLinkUrl: string;
    authCode: string;
}

export function DeepLinkRedirect({ deepLinkUrl, authCode }: DeepLinkRedirectProps) {
    useEffect(() => {
        console.log('[DeepLinkRedirect] Redirecting to:', deepLinkUrl);
        console.log('[DeepLinkRedirect] Auth code:', authCode);

        // Immediate redirect
        window.location.href = deepLinkUrl;
    }, [deepLinkUrl, authCode]);

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
            <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-8">
                <div className="text-center">
                    <div className="mb-6">
                        <svg className="mx-auto h-16 w-16 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </div>
                    <h1 className="text-2xl font-bold text-gray-900 mb-4">Authentication Successful</h1>
                    <p className="text-gray-600 mb-2">
                        Auth Code: <span className="font-mono font-bold text-blue-600">{authCode}</span>
                    </p>
                    <p className="text-gray-600 mb-6">
                        Redirecting to your desktop app...
                    </p>
                    <p className="text-sm text-gray-500 mb-4">
                        If the app doesn&apos;t open automatically, please make sure it&apos;s installed.
                    </p>
                    <a
                        href={deepLinkUrl}
                        className="inline-block bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition-colors font-medium"
                    >
                        Open Desktop App
                    </a>
                    <p className="mt-4 text-xs text-gray-400">
                        Deep link: {deepLinkUrl}
                    </p>
                </div>
            </div>
        </div>
    );
}
