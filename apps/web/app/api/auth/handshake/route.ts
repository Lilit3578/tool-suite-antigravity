
import { AuthHandshake, User } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { NextResponse } from 'next/server';
import { SignJWT } from 'jose';

export async function POST(req: Request) {
    try {
        const { otp, hardware_id } = await req.json();

        if (!otp || !hardware_id) {
            return NextResponse.json({ error: 'Missing otp or hardware_id' }, { status: 400 });
        }

        await dbConnect();

        const handshake = await AuthHandshake.findOne({ otp });

        if (!handshake) {
            return NextResponse.json({ error: 'Invalid OTP' }, { status: 401 });
        }

        if (handshake.used) {
            return NextResponse.json({ error: 'OTP already used' }, { status: 401 });
        }

        if (new Date() > handshake.expiresAt) {
            return NextResponse.json({ error: 'OTP expired' }, { status: 401 });
        }

        // Mark as used immediately to prevent race conditions (though Mongo is atomic usually)
        handshake.used = true;
        await handshake.save();

        // 3-Device Limit Check (Mocked for now as per prompt)
        const user = await User.findById(handshake.userId);
        if (!user) {
            return NextResponse.json({ error: 'User not found' }, { status: 404 });
        }

        // Ensure devices array exists
        if (!user.devices) {
            user.devices = [];
        }

        // Simple check: is this hardware_id already in devices?
        const existingDevice = user.devices.find((d: { fingerprint: string }) => d.fingerprint === hardware_id);

        if (!existingDevice) {
            if (user.devices.length >= 3) {
                // In a real app, we might block here. Logic says "Mock a check... return success for now"
                // But let's be strictly compliant with "return success for now" but also maybe add it if we can?
                // Prompt said: "Device Check: Mock a check that ensures the user has < 3 devices active (return success for now)."
                // I will just proceed.
            } else {
                // Add device
                user.devices.push({
                    fingerprint: hardware_id,
                    name: 'Desktop App', // Could pass hostname ideally
                    lastSeen: new Date()
                });
                await user.save();
            }
        } else {
            // Update last seen
            existingDevice.lastSeen = new Date();
            await user.save();
        }


        // Generate Long-Lived JWT
        // Secret should be in env. Using a fallback for dev if missing (but should be present).
        const secret = new TextEncoder().encode(
            process.env.AUTH_SECRET || process.env.JWT_SECRET || 'dev_secret_do_not_use_in_prod'
        );

        const token = await new SignJWT({
            sub: user._id.toString(),
            email: user.email,
            hwid: hardware_id
        })
            .setProtectedHeader({ alg: 'HS256' })
            .setIssuedAt()
            .setExpirationTime('30d') // 30 days session
            .sign(secret);

        return NextResponse.json({ token });

    } catch (error) {
        console.error('Handshake error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}
