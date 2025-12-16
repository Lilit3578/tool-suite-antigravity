"use server"

import { signIn } from "@/auth"
import { getUserByEmail } from "@/lib/db/queries"

export async function login(prevState: unknown, formData: FormData) {
    const email = formData.get("email") as string

    if (!email) {
        return { error: "Email is required." }
    }

    try {
        const existingUser = await getUserByEmail(email)

        if (!existingUser) {
            return { error: "No account found. Please Sign Up." }
        }

        await signIn("resend", formData, { redirectTo: "/dashboard" })
    } catch (error) {
        if (error instanceof Error) {
            // Auth.js throws a "Redirect" error on success, we need to rethrow it
            if (error.message === "NEXT_REDIRECT") {
                throw error
            }
            // Handle other Auth.js errors if needed
        }
        throw error
    }
}

export async function signup(prevState: unknown, formData: FormData) {
    const email = formData.get("email") as string

    if (!email) {
        return { error: "Email is required." }
    }

    try {
        const existingUser = await getUserByEmail(email)

        if (existingUser) {
            return { error: "Account already exists. Please Log In." }
        }

        await signIn("resend", formData, { redirectTo: "/onboarding" })
    } catch (error) {
        if (error instanceof Error) {
            // Auth.js throws a "Redirect" error on success, we need to rethrow it
            if (error.message === "NEXT_REDIRECT") {
                throw error
            }
        }
        throw error
    }
}
