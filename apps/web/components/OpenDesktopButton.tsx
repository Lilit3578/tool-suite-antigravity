
'use client';

import { useState } from 'react';

export function OpenDesktopButton() {
    const [loading, setLoading] = useState(false);

    const handleOpenDesktop = async () => {
        setLoading(true);
        try {
            const res = await fetch('/api/auth/generate-otp', {
                method: 'POST',
            });

            if (!res.ok) {
                const errorData = await res.json().catch(() => ({}));
                console.error('Failed to generate OTP:', res.status, errorData);
                alert(`Failed to generate code: ${errorData.error || res.statusText}`);
                return;
            }

            const data = await res.json();
            const { otp } = data;

            // Redirect to custom protocol
            window.location.href = `prodwidgets://auth?otp=${otp}`;

        } catch (error) {
            console.error('Error opening desktop app:', error);
            alert('Something went wrong');
        } finally {
            setLoading(false);
        }
    };

    return (
        <button
            onClick={handleOpenDesktop}
            disabled={loading}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50"
        >
            {loading ? 'Opening...' : 'Open Desktop App'}
        </button>
    );
}
