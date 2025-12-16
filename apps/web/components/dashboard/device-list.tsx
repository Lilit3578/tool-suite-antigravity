'use client';

import { revokeDevice } from '@/lib/actions/dashboard';
import { useOptimistic, useTransition } from 'react';

type Device = {
    _id: string;
    name: string;
    fingerprint: string;
    lastSeen: string;
};

export function DeviceList({ devices }: { devices: Device[] }) {
    const [isPending, startTransition] = useTransition();
    const [optimisticDevices, removeOptimisticDevice] = useOptimistic(
        devices,
        (state, deviceIdToFile: string) => state.filter((device) => device._id !== deviceIdToFile)
    );

    return (
        <div className="bg-white shadow overflow-hidden sm:rounded-md mt-6">
            <div className="px-4 py-5 sm:px-6">
                <h3 className="text-lg leading-6 font-medium text-gray-900">
                    Connected Devices
                </h3>
                <p className="mt-1 max-w-2xl text-sm text-gray-500">
                    Manage your active sessions.
                </p>
            </div>
            <ul role="list" className="divide-y divide-gray-200">
                {optimisticDevices.length === 0 ? (
                    <li className="px-4 py-4 sm:px-6 text-sm text-gray-500">
                        No devices connected.
                    </li>
                ) : (
                    optimisticDevices.map((device) => (
                        <li key={device._id} className="px-4 py-4 sm:px-6 flex items-center justify-between">
                            <div className="flex-1 min-w-0">
                                <p className="text-sm font-medium text-indigo-600 truncate">
                                    {device.name}
                                </p>
                                <div className="flex text-sm text-gray-500 mt-1">
                                    <p className="truncate mr-4">
                                        <span className="font-semibold">ID:</span> {device.fingerprint.substring(0, 8)}...
                                    </p>
                                    <p>
                                        <span className="font-semibold">Last Seen:</span> {new Date(device.lastSeen).toLocaleDateString('en-US')}
                                    </p>
                                </div>
                            </div>
                            <div className="ml-4 flex-shrink-0">
                                <button
                                    onClick={() => {
                                        startTransition(async () => {
                                            removeOptimisticDevice(device._id);
                                            await revokeDevice(device._id);
                                        });
                                    }}
                                    disabled={isPending}
                                    className="font-medium text-red-600 hover:text-red-500 disabled:opacity-50"
                                >
                                    Revoke
                                </button>
                            </div>
                        </li>
                    ))
                )}
            </ul>
        </div>
    );
}
