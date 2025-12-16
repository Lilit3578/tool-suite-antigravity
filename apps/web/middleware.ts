
import NextAuth from "next-auth"
import authConfig from "./auth.config"
import { NextResponse } from "next/server"

const { auth } = NextAuth({ ...authConfig, providers: [] })

export default auth((req) => {
    const isLoggedIn = !!req.auth
    const isOnDashboard = req.nextUrl.pathname.startsWith("/dashboard")
    const isOnOnboarding = req.nextUrl.pathname.startsWith("/onboarding")
    const isOnAuthRoute = req.nextUrl.pathname.startsWith("/api/auth")

    // Allow auth routes to always pass through (for callbacks)
    if (isOnAuthRoute) {
        return NextResponse.next()
    }

    // If trying to access protected routes but not logged in -> redirect to login
    if ((isOnDashboard || isOnOnboarding) && !isLoggedIn) {
        return NextResponse.redirect(new URL("/login", req.nextUrl))
    }

    // If logged in and on login/signup pages -> redirect to dashboard
    if (isLoggedIn && (req.nextUrl.pathname === "/login" || req.nextUrl.pathname === "/signup" || req.nextUrl.pathname === "/")) {
        return NextResponse.redirect(new URL("/dashboard", req.nextUrl))
    }

    return NextResponse.next()
})

// Optionally, don't invoke Middleware on some paths
export const config = {
    matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
}
