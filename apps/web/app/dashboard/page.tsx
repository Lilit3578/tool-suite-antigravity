
import { auth } from '@/auth';
import { User } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { redirect } from 'next/navigation';
import { DeviceList } from '@/components/dashboard/device-list';
import { OpenDesktopButton } from '@/components/OpenDesktopButton';
import Link from 'next/link';

export default async function DashboardPage() {
    const session = await auth();
    if (!session?.user?.email) {
        redirect('/login');
    }

    await dbConnect();
    const user = await User.findOne({ email: session.user.email }).lean();

    if (!user.name) {
        redirect('/onboarding');
    }

    // SERIALIZATION: Convert MongoDB specific types to primitives for client components
    const devices = (user.devices || []).map((device: any) => ({
        _id: device._id.toString(),
        name: device.name,
        fingerprint: device.fingerprint,
        lastSeen: device.lastSeen ? new Date(device.lastSeen).toISOString() : new Date().toISOString() // Ensure string
    }));

    const isPro = user.plan === 'paid';
    const deviceCount = devices.length;
    const canDownload = deviceCount < 3;

    return (
        <div className="min-h-screen bg-gray-100">
            <header className="bg-white shadow">
                <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8 flex justify-between items-center">
                    <h1 className="text-3xl font-bold text-gray-900">
                        Welcome back, {user.name}
                    </h1>
                    <form action={async () => {
                        'use server';
                        // Simple signout wrapper if needed, or link to /api/auth/signout
                        // Actually, importing signOut from auth.ts and calling it is cleaner, but server actions need "use server".
                        // For now, let's assuming a simpler logout flow or just not put it in the header right now as it wasn't strictly requested.
                    }}>
                        {/* Logout Placeholder */}
                    </form>
                </div>
            </header>
            <main>
                <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                    {/* Replace with your content */}
                    <div className="px-4 py-6 sm:px-0">

                        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 mb-8">
                            {/* Plan Card */}
                            <div className="bg-white overflow-hidden shadow rounded-lg">
                                <div className="px-4 py-5 sm:p-6">
                                    <dt className="text-sm font-medium text-gray-500 truncate">
                                        Current Plan
                                    </dt>
                                    <dd className="mt-1 text-3xl font-semibold text-gray-900">
                                        {isPro ? 'PRO' : 'Free Tier'}
                                    </dd>
                                    {!isPro && (
                                        <div className="mt-4">
                                            <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                                                Upgrade to PRO
                                            </button>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Download Card */}
                            <div className="bg-white overflow-hidden shadow rounded-lg">
                                <div className="px-4 py-5 sm:p-6">
                                    <h3 className="text-lg leading-6 font-medium text-gray-900">
                                        Download App
                                    </h3>
                                    <div className="mt-2 max-w-xl text-sm text-gray-500">
                                        <p>
                                            Get the desktop app for macOS.
                                            {deviceCount >= 3 && <span className="block text-red-500 font-bold mt-1">Device Limit Reached ({deviceCount}/3)</span>}
                                        </p>
                                    </div>
                                    <div className="mt-5">
                                        {canDownload ? (
                                            <Link
                                                href="https://example.com/download.dmg"
                                                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                            >
                                                Download for macOS
                                            </Link>
                                        ) : (
                                            <button
                                                disabled
                                                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-gray-400 cursor-not-allowed"
                                            >
                                                Download Disabled
                                            </button>
                                        )}
                                    </div>
                                </div>
                            </div>

                            {/* Desktop Integration Card */}
                            <div className="bg-white overflow-hidden shadow rounded-lg">
                                <div className="px-4 py-5 sm:p-6">
                                    <h3 className="text-lg leading-6 font-medium text-gray-900">
                                        Desktop Integration
                                    </h3>
                                    <div className="mt-2 max-w-xl text-sm text-gray-500">
                                        <p>
                                            Already have the app? Click below to securely connect your session.
                                        </p>
                                    </div>
                                    <div className="mt-5">
                                        <OpenDesktopButton />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <DeviceList devices={devices} />

                    </div>
                    {/* /End replace */}
                </div>
            </main>
        </div>
    );
}
