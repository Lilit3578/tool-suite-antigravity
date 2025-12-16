import type { NextAuthConfig } from "next-auth"
import Resend from "next-auth/providers/resend"

export default {
    providers: [
        Resend({
            from: "onboarding@resend.dev", // Default Resend testing email, user should update later
        }),
    ],
} satisfies NextAuthConfig
