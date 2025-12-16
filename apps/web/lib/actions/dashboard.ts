'use server';

import { auth } from '@/auth';
import { User } from '@/lib/db/models';
import { revalidatePath } from 'next/cache';
import { redirect } from 'next/navigation';
import dbConnect from '@/lib/db/connect';

export async function onboardUser(formData: FormData) {
    const session = await auth();
    if (!session?.user?.email) {
        throw new Error('Unauthorized');
    }

    const name = formData.get('name') as string;
    if (!name || name.trim().length === 0) {
        throw new Error('Name is required');
    }

    await dbConnect();
    await User.findOneAndUpdate(
        { email: session.user.email },
        { name: name.trim() }
    );

    redirect('/dashboard');
}

export async function revokeDevice(deviceId: string) {
    const session = await auth();
    if (!session?.user?.email) {
        throw new Error('Unauthorized');
    }

    if (!deviceId) {
        throw new Error('Device ID is required');
    }

    await dbConnect();
    const user = await User.findOne({ email: session.user.email });

    if (!user) {
        throw new Error('User not found');
    }

    // Ensure the device belongs to the user
    const deviceExists = user.devices.some((d: { _id: { toString: () => string; }; }) => d._id.toString() === deviceId);
    if (!deviceExists) {
        throw new Error('Device not found or does not belong to user');
    }

    await User.findOneAndUpdate(
        { email: session.user.email },
        { $pull: { devices: { _id: deviceId } } }
    );

    revalidatePath('/dashboard');
}
