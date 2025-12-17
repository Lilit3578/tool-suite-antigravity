import { AuthRequest, User } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { NextResponse } from 'next/server';
import { SignJWT } from 'jose';
import crypto from 'crypto';

export async function POST(req: Request) {
    try {
        const { code, verifier } = await req.json();

        if (!code || !verifier) {
            return NextResponse.json({ error: 'Missing code or verifier' }, { status: 400 });
        }

        await dbConnect();

        // Find AuthRequest by code
        const authRequest = await AuthRequest.findOne({ authCode: code });

        if (!authRequest) {
            console.error('[PKCE Exchange] Invalid auth code:', code);
            return NextResponse.json({ error: 'Invalid or expired code' }, { status: 401 });
        }

        // Check expiration
        if (new Date() > authRequest.expiresAt) {
            console.error('[PKCE Exchange] Code expired:', code);
            // Delete expired request
            await AuthRequest.deleteOne({ _id: authRequest._id });
            return NextResponse.json({ error: 'Code expired' }, { status: 401 });
        }

        // Verify PKCE challenge
        // Compute SHA256(verifier) and compare with stored challenge
        const hash = crypto.createHash('sha256').update(verifier).digest();
        const computedChallenge = hash.toString('base64url'); // URL-safe base64 without padding

        if (computedChallenge !== authRequest.challenge) {
            console.error('[PKCE Exchange] Challenge verification failed');
            console.error('[PKCE Exchange] Expected:', authRequest.challenge);
            console.error('[PKCE Exchange] Computed:', computedChallenge);
            return NextResponse.json({ error: 'Invalid verifier' }, { status: 401 });
        }

        console.log('[PKCE Exchange] ✅ Challenge verified successfully');

        // CRITICAL: Delete AuthRequest BEFORE generating token (prevent replay attacks)
        // This prioritizes security over availability (acceptable for login flow)
        await AuthRequest.deleteOne({ _id: authRequest._id });
        console.log('[PKCE Exchange] AuthRequest deleted (one-time use)');

        // Get user
        const user = await User.findById(authRequest.userId);
        if (!user) {
            console.error('[PKCE Exchange] User not found:', authRequest.userId);
            return NextResponse.json({ error: 'User not found' }, { status: 404 });
        }

        // Generate Long-Lived JWT (reuse logic from handshake)
        const secret = new TextEncoder().encode(
            process.env.AUTH_SECRET || process.env.JWT_SECRET || 'dev_secret_do_not_use_in_prod'
        );

        const token = await new SignJWT({
            sub: user._id.toString(),
            email: user.email,
        })
            .setProtectedHeader({ alg: 'HS256' })
            .setIssuedAt()
            .setExpirationTime('30d') // 30 days session
            .sign(secret);

        console.log('[PKCE Exchange] ✅ Token generated for user:', user.email);

        return NextResponse.json({ token });

    } catch (error) {
        console.error('[PKCE Exchange] CRITICAL ERROR:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}
