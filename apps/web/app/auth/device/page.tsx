import { auth } from '@/auth';
import { AuthRequest } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { redirect } from 'next/navigation';
import crypto from 'crypto';
import { DeepLinkRedirect } from '@/components/DeepLinkRedirect';

interface DeviceAuthPageProps {
    searchParams: Promise<{ challenge?: string }>;
}

export default async function DeviceAuthPage({ searchParams }: DeviceAuthPageProps) {
    const params = await searchParams;
    const challenge = params.challenge;

    // Validate challenge parameter
    if (!challenge) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-gray-50">
                <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-8">
                    <h1 className="text-2xl font-bold text-red-600 mb-4">Invalid Request</h1>
                    <p className="text-gray-700">
                        Missing challenge parameter. Please initiate login from your desktop app.
                    </p>
                </div>
            </div>
        );
    }

    // Check if user is authenticated
    const session = await auth();

    if (!session || !session.user || !session.user.email) {
        // Redirect to login with callback to return here
        const callbackUrl = `/auth/device?challenge=${encodeURIComponent(challenge)}`;
        redirect(`/login?callbackUrl=${encodeURIComponent(callbackUrl)}`);
    }

    // User is authenticated - create AuthRequest and redirect to deep link
    await dbConnect();

    // Generate random 6-digit auth code
    const authCode = crypto.randomInt(100000, 999999).toString();
    const expiresAt = new Date(Date.now() + 5 * 60 * 1000); // 5 minutes

    // Get user ID from session
    let userId = session.user.id;

    if (!userId) {
        // Fallback: Query User by email if ID is missing
        const { User } = await import('@/lib/db/models');
        const user = await User.findOne({ email: session.user.email });
        if (!user) {
            return (
                <div className="min-h-screen flex items-center justify-center bg-gray-50">
                    <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-8">
                        <h1 className="text-2xl font-bold text-red-600 mb-4">Error</h1>
                        <p className="text-gray-700">User not found in database.</p>
                    </div>
                </div>
            );
        }
        userId = user._id.toString();
    }

    // Create AuthRequest
    await AuthRequest.create({
        authCode,
        challenge,
        userId,
        expiresAt
    });

    console.log('[Device Auth] Created AuthRequest:', { authCode, userId, expiresAt });

    // Redirect to deep link using client component
    const deepLinkUrl = `prodwidgets://?code=${authCode}`;

    return <DeepLinkRedirect deepLinkUrl={deepLinkUrl} authCode={authCode} />;
}
