
import NextAuth from "next-auth"
import authConfig from "./auth.config"
import { NextResponse } from "next/server"
import { ratelimit, getClientIp } from "./lib/ratelimit"

const { auth } = NextAuth({ ...authConfig, providers: [] })

export default auth(async (req) => {
    const pathname = req.nextUrl.pathname

    // Apply rate limiting to API routes BEFORE authentication checks
    const isApiRoute = pathname.startsWith("/api/auth") ||
        pathname.startsWith("/api/translate") ||
        pathname.startsWith("/api/currency")

    if (isApiRoute) {
        const clientIp = getClientIp(req)
        const { success, limit, remaining, reset } = await ratelimit.limit(clientIp)

        if (!success) {
            return new NextResponse("Too Many Requests", {
                status: 429,
                headers: {
                    "X-RateLimit-Limit": limit.toString(),
                    "X-RateLimit-Remaining": remaining.toString(),
                    "X-RateLimit-Reset": reset.toString(),
                },
            })
        }
    }

    // Authentication and authorization logic
    const isLoggedIn = !!req.auth
    const isOnDashboard = pathname.startsWith("/dashboard")
    const isOnOnboarding = pathname.startsWith("/onboarding")
    const isOnAuthRoute = pathname.startsWith("/api/auth")

    // Allow auth routes to always pass through (for callbacks)
    if (isOnAuthRoute) {
        return NextResponse.next()
    }

    // If trying to access protected routes but not logged in -> redirect to login
    if ((isOnDashboard || isOnOnboarding) && !isLoggedIn) {
        return NextResponse.redirect(new URL("/login", req.nextUrl))
    }

    // If logged in and on login/signup pages -> redirect to dashboard
    if (isLoggedIn && (pathname === "/login" || pathname === "/signup" || pathname === "/")) {
        return NextResponse.redirect(new URL("/dashboard", req.nextUrl))
    }

    return NextResponse.next()
})

// Optionally, don't invoke Middleware on some paths
export const config = {
    matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)", "/api/:path*"],
}
