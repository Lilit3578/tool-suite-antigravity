
import { auth } from '@/auth';
import { AuthHandshake } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { NextResponse } from 'next/server';
import crypto from 'crypto';

export async function POST(req: Request) {
    try {
        const session = await auth();
        console.log('[Generate OTP] Session:', JSON.stringify(session, null, 2));

        if (!session || !session.user || !session.user.email) {
            console.error('[Generate OTP] Unauthorized: No session or email');
            return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
        }

        await dbConnect();

        // Generate 6-digit OTP
        const otp = crypto.randomInt(100000, 999999).toString();
        const expiresAt = new Date(Date.now() + 5 * 60 * 1000); // 5 minutes from now

        // We need the user's DB ID. Session usually has it if configured, or we query by email.
        // Assuming session.user.id is populated by the adapter.
        // If not, we might need to look up the user by email.
        // The MongoDB adapter usually populates `id` or `_id`.

        // Safety check: if we used a strategy that doesn't put ID in session, we fail.
        // But Auth.js with DB adapter usually does.
        // Safety check: if we used a strategy that doesn't put ID in session, we fail.
        // But Auth.js with DB adapter usually does.
        let userId = session.user.id;

        if (!userId) {
            console.log('[Generate OTP] User ID missing in session, looking up by email...');
            // Fallback: Query User by email directly if ID is missing (common in some Auth.js adapters/configs)
            const { User } = await import('@/lib/db/models');
            const user = await User.findOne({ email: session.user.email });
            if (!user) {
                console.error('[Generate OTP] User not found by email:', session.user.email);
                return NextResponse.json({ error: 'User not found in database' }, { status: 404 });
            }
            userId = user._id.toString();
            console.log('[Generate OTP] User ID found via lookup:', userId);
        } else {
            console.log('[Generate OTP] User ID present in session:', userId);
        }

        await AuthHandshake.create({
            otp,
            userId,
            expiresAt,
            used: false
        });

        return NextResponse.json({ otp, expiresAt });

    } catch (error: any) {
        console.error('[Generate OTP] CRITICAL ERROR:', error);
        return NextResponse.json({ error: error.message || 'Internal Server Error' }, { status: 500 });
    }
}
